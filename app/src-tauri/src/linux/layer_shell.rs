use std::{cell::RefCell, rc::Rc};

use anyhow::{Context, Result};
use gtk::{
  cairo::Operator,
  gdk,
  glib::Propagation,
  prelude::WidgetExtManual,
  traits::{ContainerExt, GtkWindowExt, WidgetExt},
};
use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use tauri::{App, AppHandle, Manager, async_runtime::block_on};

use crate::store::{HorizontalLayout, Settings, Store, VerticalLayout};

/// One transparent surface per monitor behind the window; a click on one
/// closes the window, like the panels of corona.
type Backdrops = Rc<RefCell<Vec<gtk::Window>>>;

pub enum GtkThreadRPC {
  ApplySettings(Settings),
  Show,
  Hide,
  Toggle,
}

pub struct GtkThreadState(async_channel::Sender<GtkThreadRPC>);

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

  gtk_window.set_namespace("positron");

  // above the backdrops, which sit on `Top` like the bars
  gtk_window.set_layer(Layer::Overlay);
  gtk_window.set_anchor(Edge::Top, settings.vertical_layout == VerticalLayout::Top);
  gtk_window.set_anchor(
    Edge::Left,
    settings.horizontal_layout == HorizontalLayout::Left,
  );
  gtk_window.set_anchor(
    Edge::Right,
    settings.horizontal_layout == HorizontalLayout::Right,
  );
  gtk_window.set_anchor(
    Edge::Bottom,
    settings.vertical_layout == VerticalLayout::Bottom,
  );

  gtk_window.set_width_request(settings.width as i32);
  gtk_window.set_height_request(settings.height as i32);

  gtk_window.set_keyboard_mode(KeyboardMode::OnDemand);

  #[cfg(not(debug_assertions))]
  gtk_window.hide();
  #[cfg(debug_assertions)]
  gtk_window.show_all();

  let (sender, receiver) = async_channel::bounded::<GtkThreadRPC>(10);

  let backdrops = Backdrops::default();
  gtk::glib::spawn_future_local(async move {
    while let Ok(msg) = receiver.recv().await {
      handle_rpc(msg, &gtk_window, &backdrops);
    }
  });

  app.manage(GtkThreadState(sender));

  Ok(())
}

pub async fn send_rpc(app: &AppHandle, msg: GtkThreadRPC) {
  let gtk_thread_state = app.state::<GtkThreadState>();
  gtk_thread_state.0.send(msg).await.ok();
}

pub fn handle_rpc(msg: GtkThreadRPC, gtk_window: &gtk::ApplicationWindow, backdrops: &Backdrops) {
  match msg {
    GtkThreadRPC::ApplySettings(settings) => {
      gtk_window.set_anchor(Edge::Top, settings.vertical_layout == VerticalLayout::Top);
      gtk_window.set_anchor(
        Edge::Left,
        settings.horizontal_layout == HorizontalLayout::Left,
      );
      gtk_window.set_anchor(
        Edge::Right,
        settings.horizontal_layout == HorizontalLayout::Right,
      );
      gtk_window.set_anchor(
        Edge::Bottom,
        settings.vertical_layout == VerticalLayout::Bottom,
      );

      gtk_window.set_width_request(settings.width as i32);
      gtk_window.set_height_request(settings.height as i32);
    }
    GtkThreadRPC::Show => show(gtk_window, backdrops),
    GtkThreadRPC::Hide => hide(gtk_window, backdrops),
    GtkThreadRPC::Toggle => {
      if gtk_window.is_visible() {
        hide(gtk_window, backdrops);
      } else {
        show(gtk_window, backdrops);
      }
    }
  }
}

fn show(window: &gtk::ApplicationWindow, backdrops: &Backdrops) {
  if window.is_visible() {
    return;
  }

  // monitors come and go, so the backdrops are made per show
  if let Some(display) = gdk::Display::default() {
    let mut list = backdrops.borrow_mut();
    for i in 0..display.n_monitors() {
      let Some(monitor) = display.monitor(i) else {
        continue;
      };
      let (window, backdrops) = (window.clone(), backdrops.clone());
      let backdrop = backdrop(&monitor, move || hide(&window, &backdrops));
      backdrop.show_all();
      list.push(backdrop);
    }
  }

  window.show_all();
}

fn hide(window: &gtk::ApplicationWindow, backdrops: &Backdrops) {
  window.hide();

  let backdrops: Vec<_> = backdrops.borrow_mut().drain(..).collect();
  // this can run in the click handler of a backdrop, which must not
  // be destroyed while it handles the event
  gtk::glib::idle_add_local_once(move || {
    for backdrop in backdrops {
      backdrop.close();
    }
  });
}

/// A transparent surface over `monitor`, taking no keyboard focus. It leaves
/// the exclusive zones of the bars alone, so those stay usable.
fn backdrop(monitor: &gdk::Monitor, on_click: impl Fn() + 'static) -> gtk::Window {
  let window = gtk::Window::new(gtk::WindowType::Toplevel);

  window.set_app_paintable(true);
  if let Some(visual) = WidgetExt::screen(&window).and_then(|screen| screen.rgba_visual()) {
    window.set_visual(Some(&visual));
  }
  // GTK would paint the theme's window background otherwise
  window.connect_draw(|_, cr| {
    cr.set_operator(Operator::Clear);
    let _ = cr.paint();
    Propagation::Stop
  });

  window.init_layer_shell();
  window.set_namespace("positron_backdrop");
  window.set_layer(Layer::Top);
  window.set_monitor(monitor);
  for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
    window.set_anchor(edge, true);
  }
  window.set_keyboard_mode(KeyboardMode::None);

  window.add_events(gdk::EventMask::BUTTON_PRESS_MASK);
  window.connect_button_press_event(move |_, _| {
    on_click();
    Propagation::Stop
  });

  window
}
