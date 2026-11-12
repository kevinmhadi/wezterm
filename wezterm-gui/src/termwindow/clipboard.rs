use crate::termwindow::TermWindowNotif;
use crate::TermWindow;
use config::keyassignment::{ClipboardCopyDestination, ClipboardPasteSource};
use mux::pane::{Pane, PaneId};
use mux::Mux;
use std::sync::Arc;
use termwiz::escape::osc::{OperatingSystemCommand, Selection};
use wezterm_term::ClipboardSelection;
use window::{Clipboard, WindowOps};

impl TermWindow {
    pub fn read_clipboard_for_osc52(&mut self, pane_id: PaneId, selection: ClipboardSelection) {
        // Enforce the policy in the GUI, even when the query originated on a
        // remote mux server. Never read another window's clipboard on its behalf.
        if !self.config.allow_osc52_clipboard_read || !self.window_contains_pane(pane_id) {
            return;
        }

        let (clipboard, selection) = match selection {
            ClipboardSelection::Clipboard => (Clipboard::Clipboard, Selection::CLIPBOARD),
            ClipboardSelection::PrimarySelection => {
                (Clipboard::PrimarySelection, Selection::PRIMARY)
            }
        };
        let window = self.window.as_ref().unwrap().clone();
        // These backends return a ready future after synchronous OS IPC.
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        let future = {
            let window = window.clone();
            smol::unblock(move || promise::spawn::block_on(window.get_clipboard(clipboard)))
        };
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let future = window.get_clipboard(clipboard);
        promise::spawn::spawn(async move {
            match future.await {
                Ok(clip) => {
                    window.notify(TermWindowNotif::Apply(Box::new(move |myself| {
                        // Permission or pane ownership may have changed while
                        // the platform clipboard request was pending.
                        if !myself.config.allow_osc52_clipboard_read
                            || !myself.window_contains_pane(pane_id)
                        {
                            return;
                        }
                        if let Some(pane) = Mux::get().get_pane(pane_id) {
                            promise::spawn::spawn(async move {
                                // Both a full PTY and a remote pane writer can
                                // block. Encode and write the raw response off
                                // the GUI thread, without paste transformations.
                                let result = smol::unblock(move || -> anyhow::Result<()> {
                                    let response =
                                        OperatingSystemCommand::SetSelection(selection, clip)
                                            .to_string();
                                    let mut writer = pane.writer();
                                    writer.write_all(response.as_bytes())?;
                                    writer.flush()?;
                                    Ok(())
                                })
                                .await;
                                if let Err(err) = result {
                                    log::error!("OSC 52 reply for pane {pane_id}: {err:#}");
                                }
                            })
                            .detach();
                        }
                    })));
                }
                Err(err) => log::error!("OSC 52 clipboard read for pane {pane_id}: {err:#}"),
            }
        })
        .detach();
    }

    pub fn copy_to_clipboard(&self, clipboard: ClipboardCopyDestination, text: String) {
        let clipboard = match clipboard {
            ClipboardCopyDestination::Clipboard => [Some(Clipboard::Clipboard), None],
            ClipboardCopyDestination::PrimarySelection => [Some(Clipboard::PrimarySelection), None],
            ClipboardCopyDestination::ClipboardAndPrimarySelection => [
                Some(Clipboard::Clipboard),
                Some(Clipboard::PrimarySelection),
            ],
        };
        for &c in &clipboard {
            if let Some(c) = c {
                self.window.as_ref().unwrap().set_clipboard(c, text.clone());
            }
        }
    }

    pub fn paste_from_clipboard(&mut self, pane: &Arc<dyn Pane>, clipboard: ClipboardPasteSource) {
        let pane_id = pane.pane_id();
        log::trace!(
            "paste_from_clipboard in pane {} {:?}",
            pane.pane_id(),
            clipboard
        );
        let window = self.window.as_ref().unwrap().clone();
        let clipboard = match clipboard {
            ClipboardPasteSource::Clipboard => Clipboard::Clipboard,
            ClipboardPasteSource::PrimarySelection => Clipboard::PrimarySelection,
        };
        let future = window.get_clipboard(clipboard);
        promise::spawn::spawn(async move {
            if let Ok(clip) = future.await {
                window.notify(TermWindowNotif::Apply(Box::new(move |myself| {
                    if let Some(pane) = myself
                        .pane_state(pane_id)
                        .overlay
                        .as_ref()
                        .map(|overlay| overlay.pane.clone())
                        .or_else(|| {
                            let mux = Mux::get();
                            mux.get_pane(pane_id)
                        })
                    {
                        pane.send_paste(&clip).ok();
                    }
                })));
            }
        })
        .detach();
        self.maybe_scroll_to_bottom_for_input(&pane);
    }
}
