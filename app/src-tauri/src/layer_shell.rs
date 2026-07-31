use anyhow::{Context, Result};
use gtk::traits::{ContainerExt, GtkWindowExt, WidgetExt};
use gtk_layer_shell::LayerShell;
use tauri::{App, Manager, async_runtime::block_on};

use crate::store::{HorizontalLayout, Store, VerticalLayout};

pub fn init(app: &App) -> Result<()> {
  let store = app.state::<Store>();
  let settings = block_on(store.settings());

  let main_window = app
    .get_webview_window("main")
    .context("Main window missing")?;
  main_window.hide()?;

  let gtk_window = gtk::ApplicationWindow::new(
    &main_window
      .gtk_window()?
      .application()
      .context("Failed to get gtk application")?,
  );

  gtk_window.set_app_paintable(true);

  let vbox = main_window.default_vbox()?;
  main_window.gtk_window()?.remove(&vbox);
  gtk_window.add(&vbox);

  gtk_window.init_layer_shell();

  gtk_window.set_layer(gtk_layer_shell::Layer::Top);
  gtk_window.set_anchor(
    gtk_layer_shell::Edge::Top,
    settings.vertical_layout == VerticalLayout::Top,
  );
  gtk_window.set_anchor(
    gtk_layer_shell::Edge::Left,
    settings.horizontal_layout == HorizontalLayout::Left,
  );
  gtk_window.set_anchor(
    gtk_layer_shell::Edge::Right,
    settings.horizontal_layout == HorizontalLayout::Right,
  );
  gtk_window.set_anchor(
    gtk_layer_shell::Edge::Bottom,
    settings.vertical_layout == VerticalLayout::Bottom,
  );

  gtk_window.set_width_request(800);
  gtk_window.set_height_request(400);

  gtk_window.set_keyboard_mode(gtk_layer_shell::KeyboardMode::OnDemand);

  #[cfg(not(debug_assertions))]
  gtk_window.hide();
  #[cfg(debug_assertions)]
  gtk_window.show_all();

  Ok(())
}
