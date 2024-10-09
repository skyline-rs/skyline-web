#![feature(new_uninit)]
#![feature(new_zeroed_alloc)]

use std::collections::HashMap;
use std::ffi::CStr;
use std::fs;
use std::num::NonZeroU32;
use std::path::Path;
use std::str::Utf8Error;

use skyline::info::get_program_id;

use nnsdk::web::offlinewebsession::*;
use nnsdk::web::*;
use skyline::nn::os::{SystemEventClearMode, SystemEventType, TryWaitSystemEvent};

pub use nnsdk::web::{
    offlinewebsession::OfflineWebSession, OfflineBackgroundKind as Background,
    OfflineBootDisplayKind as BootDisplay, WebSessionBootMode as Visibility,
};

mod session;
pub use session::WebSession;

pub struct PageResult {
    ret: Box<OfflineHtmlPageReturnValue>,
}

impl PageResult {
    pub fn new() -> Self {
        let mut ret;
        unsafe {
            ret = Box::<OfflineHtmlPageReturnValue>::new_zeroed().assume_init();

            OfflineHtmlPageReturnValue(ret.as_mut());
        }

        PageResult { ret }
    }

    pub fn get_last_url(&self) -> Result<&str, Utf8Error> {
        unsafe {
            let last_url = GetLastUrl(self.ret.as_ref());
            CStr::from_ptr(last_url as _).to_str()
        }
    }

    pub fn get_exit_reason(&self) -> OfflineExitReason {
        self.ret.get_exit_reason()
    }
}

impl AsRef<OfflineHtmlPageReturnValue> for PageResult {
    fn as_ref(&self) -> &OfflineHtmlPageReturnValue {
        &self.ret
    }
}

impl AsMut<OfflineHtmlPageReturnValue> for PageResult {
    fn as_mut(&mut self) -> &mut OfflineHtmlPageReturnValue {
        &mut self.ret
    }
}

pub struct Webpage<'a> {
    files: HashMap<&'a str, &'a [u8]>,
    dir: Option<&'a Path>,
    show: Option<&'a str>,
    htdocs_dir: Option<&'a str>,
    background: OfflineBackgroundKind,
    boot_display: OfflineBootDisplayKind,
    javascript: bool,
    footer: bool,
    pointer: bool,
    boot_icon: bool,
    web_audio: bool,
}

impl<'a> Default for Webpage<'a> {
    fn default() -> Self {
        Self {
            files: HashMap::new(),
            dir: None,
            show: None,
            htdocs_dir: None,
            background: OfflineBackgroundKind::Default,
            boot_display: OfflineBootDisplayKind::Default,
            javascript: true,
            footer: false,
            pointer: false,
            boot_icon: false,
            web_audio: true,
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct OsError(NonZeroU32);

impl<'a> Webpage<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a single file to the context of the webpage
    pub fn file<S, D>(&mut self, name: &'a S, data: &'a D) -> &mut Self
    where
        S: AsRef<str> + ?Sized + 'a,
        D: AsRef<[u8]> + ?Sized + 'a,
    {
        self.files.insert(name.as_ref(), data.as_ref());

        self
    }

    pub fn files<Str, Data, Arr>(&mut self, files: &'a Arr) -> &mut Self
    where
        Str: AsRef<str> + Sized + 'a,
        Data: AsRef<[u8]> + Sized + 'a,
        Arr: AsRef<[(Str, Data)]> + ?Sized + 'a,
    {
        for (name, data) in files.as_ref().into_iter() {
            self.files.insert(name.as_ref(), data.as_ref());
        }

        self
    }

    pub fn with_dir<P>(&mut self, dir_path: &'a P) -> &mut Self
    where
        P: AsRef<Path> + ?Sized + 'a,
    {
        self.dir = Some(dir_path.as_ref());

        self
    }

    pub fn background(&mut self, bg: OfflineBackgroundKind) -> &mut Self {
        self.background = bg;

        self
    }

    pub fn boot_display(&mut self, boot: OfflineBootDisplayKind) -> &mut Self {
        self.boot_display = boot;

        self
    }

    pub fn javascript(&mut self, js: bool) -> &mut Self {
        self.javascript = js;

        self
    }

    pub fn footer(&mut self, footer: bool) -> &mut Self {
        self.footer = footer;

        self
    }

    pub fn pointer(&mut self, pointer: bool) -> &mut Self {
        self.pointer = pointer;

        self
    }

    pub fn boot_icon(&mut self, boot_icon: bool) -> &mut Self {
        self.boot_icon = boot_icon;

        self
    }

    pub fn web_audio(&mut self, audio: bool) -> &mut Self {
        self.web_audio = audio;

        self
    }

    pub fn start_page<S>(&mut self, path: &'a S) -> &mut Self
    where
        S: AsRef<str> + ?Sized + 'a,
    {
        self.show = Some(path.as_ref());

        self
    }

    pub fn htdocs_dir<S>(&mut self, path: &'a S) -> &mut Self
    where
        S: AsRef<str> + ?Sized + 'a,
    {
        self.htdocs_dir = Some(path.as_ref());

        self
    }

    fn into_page_args(&mut self) -> Result<Box<ShowOfflineHtmlPageArg>, OsError> {
        let program_id = get_program_id();

        let htdocs_dir = self.htdocs_dir.unwrap_or("temp");

        let folder_path = Path::new("sd:/atmosphere/contents")
            .join(&format!("{:016X}", program_id))
            .join(&format!("manual_html/html-document/{}.htdocs/", htdocs_dir));

        if let Some(dir) = self.dir {
            // Copy dir to temp.htdocs
        } else if !folder_path.exists() {
            let _ = fs::create_dir_all(&folder_path);
        }

        for (path, data) in self.files.iter() {
            let file_path = folder_path.join(path);
            let file_parent = file_path.parent().unwrap();
            if !file_parent.exists() {
                fs::create_dir_all(file_parent).unwrap();
            }
            fs::write(file_path, data).unwrap();
        }

        let mut args = new_boxed_html_page_arg(format!(
            "{}.htdocs/{}",
            htdocs_dir,
            self.show.unwrap_or("index.html")
        ));

        args.set_background_kind(self.background);
        args.set_boot_display_kind(self.boot_display);
        args.enable_javascript(self.javascript);
        args.display_footer(self.footer);
        args.enable_pointer(self.pointer);
        args.enable_boot_loading_icon(self.boot_icon);
        args.enable_web_audio(self.web_audio);

        Ok(args)
    }

    pub fn open_session(&mut self, boot_mode: Visibility) -> Result<WebSession, OsError> {
        self.javascript(true);

        let mut args = self.into_page_args()?;
        args.set_boot_mode(boot_mode);

        let session = OfflineWebSession::new();
        let system_evt = SystemEventType::new(SystemEventClearMode::Manual);

        unsafe {
            Start(&session, &&system_evt, &args);
            TryWaitSystemEvent(&system_evt);
        }

        Ok(WebSession(session))
    }

    pub fn open(&mut self) -> Result<PageResult, OsError> {
        let mut args = self.into_page_args().unwrap();
        let mut page_result = PageResult::new();

        let result = unsafe { ShowOfflineHtmlPage(page_result.as_mut(), args.as_mut()) };

        match result {
            0 => Ok(page_result),
            err => Err(OsError(NonZeroU32::new(err).unwrap())),
        }
    }
}

fn new_boxed_html_page_arg<T>(page_path: T) -> Box<ShowOfflineHtmlPageArg>
where
    T: AsRef<[u8]>,
{
    let mut path_bytes = page_path.as_ref().to_vec();

    if path_bytes.len() > 3072 {
        path_bytes.truncate(3071);
    }

    path_bytes.push(b'\0');

    unsafe {
        let mut instance = Box::<ShowOfflineHtmlPageArg>::new_zeroed().assume_init();
        ShowOfflineHtmlPageArg(instance.as_mut(), path_bytes.as_ptr());
        instance
    }
}

pub mod dialog;
pub mod dialog_ok;
pub extern crate ramhorns;
pub use ramhorns::*;
