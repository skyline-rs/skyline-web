use crate::{Background, BootDisplay, Webpage};
use ramhorns::{Template, Content};

#[derive(Content)]
pub struct DialogOk {
    #[md]
    text: String,
    ok_button: String,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DialogOption {
    Ok,
}

impl DialogOk {
    pub fn new<S1, S2>(text: S1, ok_button: S2) -> Self
        where S1: Into<String>,
              S2: Into<String>,
    {
        Self {
            text: text.into(),
            ok_button: ok_button.into(),
        }
    }

    pub fn ok<S: Into<String>>(message: S) -> bool {
        match DialogOk::new(message, "OK").show() {
            DialogOption::Ok => true,
        }
    }

    pub fn show(&self) -> DialogOption {
        let tpl = Template::new(include_str!("templates/dialog_ok.html")).unwrap();

        let response = Webpage::new()
            .background(Background::BlurredScreenshot)
            .file("index.html", &tpl.render(self))
            .boot_display(BootDisplay::BlurredScreenshot)
            .open()
            .unwrap();

        DialogOption::Ok
    }
}
