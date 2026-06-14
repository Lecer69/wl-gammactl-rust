use anyhow::{Context, Result};
use std::os::fd::{AsFd, FromRawFd, OwnedFd};
use tempfile::tempfile;
use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_output, wl_registry},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols_wlr::gamma_control::v1::client::{
    zwlr_gamma_control_manager_v1, zwlr_gamma_control_v1,
};

use crate::gamma::{fill_table, Params};

struct Output {
    wl: wl_output::WlOutput,
    ctrl: Option<zwlr_gamma_control_v1::ZwlrGammaControlV1>,
    ramp_size: u32,
}

pub struct WaylandState {
    manager: zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1,
    outputs: Vec<Output>,
}

impl WaylandState {
    pub fn connect() -> Result<(Self, Connection, wayland_client::EventQueue<Self>)> {
        let conn = Connection::connect_to_env().context("cannot connect to Wayland display")?;
        let (globals, mut queue) = registry_queue_init::<Self>(&conn)?;
        let qh = queue.handle();

        let manager = globals
            .bind::<zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1, _, _>(&qh, 1..=1, ())
            .context("compositor doesn't support wlr-gamma-control-unstable-v1")?;

        let mut state = WaylandState {
            manager,
            outputs: Vec::new(),
        };

        for g in globals.contents().clone_list() {
            if g.interface == "wl_output" {
                let wl = globals.registry().bind::<wl_output::WlOutput, _, _>(
                    g.name,
                    g.version.min(4),
                    &qh,
                    (),
                );
                state.outputs.push(Output {
                    wl,
                    ctrl: None,
                    ramp_size: 0,
                });
            }
        }

        let wl_outputs: Vec<wl_output::WlOutput> =
            state.outputs.iter().map(|o| o.wl.clone()).collect();

        for (i, wl) in wl_outputs.iter().enumerate() {
            let ctrl = state.manager.get_gamma_control(wl, &qh, i);
            if let Some(out) = state.outputs.get_mut(i) {
                out.ctrl = Some(ctrl);
            }
        }

        queue.roundtrip(&mut state)?;

        Ok((state, conn, queue))
    }

    pub fn apply(
        &mut self,
        params: &Params,
        queue: &mut wayland_client::EventQueue<Self>,
    ) -> Result<()> {
        for out in &self.outputs {
            let Some(ctrl) = &out.ctrl else { continue };
            if out.ramp_size == 0 {
                continue;
            }

            let mut table = vec![0u16; out.ramp_size as usize * 3];
            fill_table(&mut table, out.ramp_size, params);
            ctrl.set_gamma(table_to_fd(&table)?.as_fd());
        }
        queue.roundtrip(self)?;
        Ok(())
    }
}

fn table_to_fd(table: &[u16]) -> Result<OwnedFd> {
    use std::io::{Seek, SeekFrom, Write};
    use std::os::unix::io::IntoRawFd;

    let mut f = tempfile()?;
    let bytes = unsafe { std::slice::from_raw_parts(table.as_ptr() as *const u8, table.len() * 2) };
    f.write_all(bytes)?;
    f.seek(SeekFrom::Start(0))?;
    Ok(unsafe { OwnedFd::from_raw_fd(f.into_raw_fd()) })
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandState {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_output::WlOutput, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &wl_output::WlOutput,
        _: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1,
        _: zwlr_gamma_control_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_gamma_control_v1::ZwlrGammaControlV1, usize> for WaylandState {
    fn event(
        state: &mut Self,
        _ctrl: &zwlr_gamma_control_v1::ZwlrGammaControlV1,
        event: zwlr_gamma_control_v1::Event,
        idx: &usize,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_gamma_control_v1::Event::GammaSize { size } => {
                if let Some(out) = state.outputs.get_mut(*idx) {
                    out.ramp_size = size;
                }
            }
            zwlr_gamma_control_v1::Event::Failed => {
                eprintln!("gamma control failed for output {idx}");
                if let Some(out) = state.outputs.get_mut(*idx) {
                    out.ctrl = None;
                }
            }
            _ => {}
        }
    }
}
