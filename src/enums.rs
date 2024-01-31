use super::utils::to_uppercase;

use std::io;
use std::path::{Path, PathBuf};
use std::str;


#[derive(Clone, Debug)]
pub enum Command {
    Auth,
    Syst,
    User(String),
    Cwd(PathBuf),
    Pwd,
    Noop,
    Type,
    Pasv,
    Unknown(String),
}

impl AsRef<str> for Command {
    fn as_ref(&self) -> &str {
        match *self {
            Self::Auth => "AUTH",
            Self::Syst => "SYST",
            Self::User(_) => "USER",
            Self::Noop => "NOOP",
            Self::Cwd(_) => "CWD",
            Self::Pwd => "PWD",
            Self::Type => "TYPE",
            Self::Pasv => "PASV",
            Self::Unknown(_) => "UNKN",
        }
    }
}

impl Command {
    pub fn new(input: Vec<u8>) -> io::Result<Self> {
        let mut iter = input.split(|&byte| byte == b' ');
        let mut command = iter.next().expect("Command in input").to_vec();
        to_uppercase(&mut command);
        let data = iter.next();
        let command = match command.as_slice() {
            b"AUTH" => Command::Auth,
            b"CWD" => Command::Cwd(data.map(|bytes|
                Path::new(str::from_utf8(bytes).unwrap()).to_path_buf()).unwrap()),
            b"SYST" => Command::Syst,
            b"NOOP" => Command::Noop,
            b"PWD" => Command::Pwd,
            b"TYPE" => Command::Type,
            b"PASV" => Command::Pasv,
            b"USER" => Command::User(data.map(|bytes| String::from_utf8(bytes.to_vec())
                .expect("cannot convert  bytes to string")).unwrap_or_default()),
            s => Command::Unknown(str::from_utf8(s).unwrap_or("").to_owned()),
        };
        Ok(command)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
#[allow(dead_code)]
pub enum ResultCode {
    RestartMarkerReply = 110,
    ServiceReadInXXXMinutes = 120,
    DataConnectionAlreadyOpen = 125,
    FileStatusOk = 150,
    Ok = 200,
    CommandNotImplementedSuperfluousAtThisSite = 202,
    SystemStatus = 211,
    DirectoryStatus = 212,
    FileStatus = 213,
    HelpMessage = 214,
    SystemType = 215,
    ServiceReadyForNewUser = 220,
    ServiceClosingControlConnection = 221,
    DataConnectionOpen = 225,
    ClosingDataConnection = 226,
    EnteringPassiveMode = 227,
    UserLoggedIn = 230,
    RequestedFileActionOkay = 250,
    PATHNAMECreated = 257,
    UserNameOkayNeedPassword = 331,
    NeedAccountForLogin = 332,
    RequestedFileActionPendingFurtherInformation = 350,
    ServiceNotAvailable = 421,
    CantOpenDataConnection = 425,
    ConnectionClosed = 426,
    FileBusy = 450,
    LocalErrorInProcessing = 451,
    InsufficientStorageSpace = 452,
    UnknownCommand = 500,
    InvalidParameterOrArgument = 501,
    CommandNotImplemented = 502,
    BadSequenceOfCommands = 503,
    CommandNotImplementedForThatParameter = 504,
    NotLoggedIn = 530,
    NeedAccountForStoringFiles = 532,
    FileNotFound = 550,
    PageTypeUnknown = 551,
    ExceededStorageAllocation = 552,
    FileNameNotAllowed = 553,
}
