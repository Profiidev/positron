pub fn confirm_code(code: &String, old: bool, link: &String) -> String {
  let old = if old { "old" } else { "new" };

  format!(
    r#"
  <!DOCTYPE html>
  <html lang="en">
    <head>
      <meta charset="UTF-8">
      <meta name="viewport" content="width=device-width, initial-scale=1.0">
      <title>Confirm Code</title>
    </head>
    <body>
      <div style="display: flex; flex-direction: column;">
        <header style="padding: 1rem; display: flex; flex-direction: column; align-items: center; justify-content: center;">
          <h2 style="margin: 0;">Confirm Code</h2>
          <p style="margin: 0;">Enter this code on the website to confirm that this is your {} email</p>
        </header>
        <div style="display: flex; align-items: center; justify-content: center;">
          <h3>{}</h3>
        </div>
        <footer style="display: flex; align-items: center; justify-content: center;">
          <p>Mail send from <a href="{}">{}</a></p>
        </footer>
      </div>
    </body>
  </html>
  "#,
    old, code, link, link
  )
}
