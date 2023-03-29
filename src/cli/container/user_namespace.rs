use std::{num::ParseIntError, str::FromStr};

use thiserror::Error;

use super::Output;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Auto {
        uidmapping: Option<String>,
        gidmapping: Option<String>,
        size: Option<u32>,
    },
    Container {
        id: String,
    },
    Host,
    KeepId {
        uid: Option<u32>,
        gid: Option<u32>,
    },
    Nomap,
    Ns {
        namespace: String,
    },
}

impl FromStr for Mode {
    type Err = ParseModeError;

    #[allow(clippy::similar_names)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s.starts_with("auto") => s.split_once(':').map_or(
                Ok(Mode::Auto {
                    uidmapping: None,
                    gidmapping: None,
                    size: None,
                }),
                |(_, options)| {
                    let mut uidmapping = None;
                    let mut gidmapping = None;
                    let mut size = None;
                    for option in options.split(',') {
                        if option.starts_with("uidmapping=") {
                            let (_, option) =
                                option.split_once('=').expect("delimiter is in guard");
                            uidmapping = Some(String::from(option));
                        } else if option.starts_with("gidmapping=") {
                            let (_, option) =
                                option.split_once('=').expect("delimiter is in guard");
                            gidmapping = Some(String::from(option));
                        } else if option.starts_with("size=") {
                            let (_, option) =
                                option.split_once('=').expect("delimiter is in guard");
                            size = Some(option.parse().map_err(|source| {
                                ParseModeError::AutoSizeParseError {
                                    size: String::from(option),
                                    source,
                                }
                            })?);
                        } else {
                            return Err(ParseModeError::InvalidAutoOption(String::from(option)));
                        }
                    }
                    Ok(Mode::Auto {
                        uidmapping,
                        gidmapping,
                        size,
                    })
                },
            ),
            s if s.starts_with("container:") => {
                let (_, id) = s.split_once(':').expect("delimiter is in guard");
                Ok(Mode::Container {
                    id: String::from(id),
                })
            }
            "host" => Ok(Mode::Host),
            s if s.starts_with("keep-id") => s.split_once(':').map_or(
                Ok(Mode::KeepId {
                    uid: None,
                    gid: None,
                }),
                |(_, options)| {
                    let mut uid = None;
                    let mut gid = None;
                    for option in options.split(',') {
                        if option.starts_with("uid=") {
                            let (_, option) =
                                option.split_once('=').expect("delimiter is in guard");
                            uid = Some(option.parse().map_err(|source| {
                                ParseModeError::KeepIdUidParseError {
                                    uid: String::from(option),
                                    source,
                                }
                            })?);
                        } else if option.starts_with("gid=") {
                            let (_, option) =
                                option.split_once('=').expect("delimiter is in guard");
                            gid = Some(option.parse().map_err(|source| {
                                ParseModeError::KeepIdGidParseError {
                                    gid: String::from(option),
                                    source,
                                }
                            })?);
                        } else {
                            return Err(ParseModeError::InvalidKeepIdOption(String::from(option)));
                        }
                    }
                    Ok(Mode::KeepId { uid, gid })
                },
            ),
            "nomap" => Ok(Mode::Nomap),
            s if s.starts_with("ns:") => {
                let (_, namespace) = s.split_once(':').expect("delimiter is in guard");
                Ok(Mode::Ns {
                    namespace: String::from(namespace),
                })
            }
            _ => Err(ParseModeError::InvalidMode(String::from(s))),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseModeError {
    #[error("`{0}` is not a valid auto mode option")]
    InvalidAutoOption(String),

    #[error("`{size}` is not a valid size: {source}")]
    AutoSizeParseError { size: String, source: ParseIntError },

    #[error("`{0}` is not a valid keep-id mode option")]
    InvalidKeepIdOption(String),

    #[error("`{uid}` is not a valid UID: {source}")]
    KeepIdUidParseError { uid: String, source: ParseIntError },

    #[error("`{gid}` is not a valid GID: {source}")]
    KeepIdGidParseError { gid: String, source: ParseIntError },

    #[error("`{0}` is not a valid user namespace mode")]
    InvalidMode(String),
}

impl From<Mode> for Output {
    fn from(value: Mode) -> Self {
        (&value).into()
    }
}

impl From<&Mode> for Output {
    fn from(value: &Mode) -> Self {
        match value {
            Mode::Auto {
                uidmapping,
                gidmapping,
                size,
            } => {
                let mut options = vec![String::from("RemapUsers=auto")];
                if let Some(uidmapping) = uidmapping {
                    options.push(format!("RemapUid={uidmapping}"));
                }
                if let Some(gidmapping) = gidmapping {
                    options.push(format!("RemapGid={gidmapping}"));
                }
                if let Some(size) = size {
                    options.push(format!("RemapUidSize={size}"));
                }
                Self::QuadletOptions(options.join("\n"))
            }
            Mode::Container { id } => Self::PodmanArg(format!("container:{id}")),
            Mode::Host => Self::PodmanArg(String::from("host")),
            Mode::KeepId { uid, gid } => {
                if uid.is_some() || gid.is_some() {
                    let mut options = Vec::new();
                    if let Some(uid) = uid {
                        options.push(format!("uid={uid}"));
                    }
                    if let Some(gid) = gid {
                        options.push(format!("gid={gid}"));
                    }
                    Self::PodmanArg(format!("keep-id:{}", options.join(",")))
                } else {
                    Self::QuadletOptions(String::from("RemapUsers=keep-id"))
                }
            }
            Mode::Nomap => Self::PodmanArg(String::from("nomap")),
            Mode::Ns { namespace } => Self::PodmanArg(format!("ns:{namespace}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse {
        use super::*;

        #[test]
        fn auto_no_options() {
            let sut = "auto".parse();
            assert_eq!(
                sut,
                Ok(Mode::Auto {
                    uidmapping: None,
                    gidmapping: None,
                    size: None
                })
            );
        }

        #[test]
        fn auto_all_options() {
            let sut = "auto:uidmapping=100,gidmapping=200,size=300".parse();
            assert_eq!(
                sut,
                Ok(Mode::Auto {
                    uidmapping: Some(String::from("100")),
                    gidmapping: Some(String::from("200")),
                    size: Some(300)
                })
            );
        }

        #[test]
        fn container() {
            let sut = "container:name".parse();
            assert_eq!(
                sut,
                Ok(Mode::Container {
                    id: String::from("name")
                })
            );
        }

        #[test]
        fn host() {
            let sut = "host".parse();
            assert_eq!(sut, Ok(Mode::Host));
        }

        #[test]
        fn keep_id_no_options() {
            let sut = "keep-id".parse();
            assert_eq!(
                sut,
                Ok(Mode::KeepId {
                    uid: None,
                    gid: None
                })
            );
        }

        #[test]
        fn keep_id_all_options() {
            let sut = "keep-id:uid=100,gid=200".parse();
            assert_eq!(
                sut,
                Ok(Mode::KeepId {
                    uid: Some(100),
                    gid: Some(200)
                })
            );
        }

        #[test]
        fn nomap() {
            let sut = "nomap".parse();
            assert_eq!(sut, Ok(Mode::Nomap));
        }

        #[test]
        fn ns() {
            let sut = "ns:namespace".parse();
            assert_eq!(
                sut,
                Ok(Mode::Ns {
                    namespace: String::from("namespace")
                })
            );
        }

        #[test]
        fn invalid_auto_option() {
            let sut: Result<Mode, ParseModeError> = "auto:".parse();
            assert_eq!(sut, Err(ParseModeError::InvalidAutoOption(String::new())));
        }

        #[test]
        fn auto_size_parse_error() {
            let sut: Result<Mode, ParseModeError> = "auto:size=".parse();
            assert!(matches!(
                sut,
                Err(ParseModeError::AutoSizeParseError { .. })
            ));
        }

        #[test]
        fn invalid_keep_id_option() {
            let sut: Result<Mode, ParseModeError> = "keep-id:".parse();
            assert_eq!(sut, Err(ParseModeError::InvalidKeepIdOption(String::new())));
        }

        #[test]
        fn keep_id_uid_parse_error() {
            let sut: Result<Mode, ParseModeError> = "keep-id:uid=".parse();
            assert!(matches!(
                sut,
                Err(ParseModeError::KeepIdUidParseError { .. })
            ));
        }

        #[test]
        fn keep_id_gid_parse_error() {
            let sut: Result<Mode, ParseModeError> = "keep-id:gid=".parse();
            assert!(matches!(
                sut,
                Err(ParseModeError::KeepIdGidParseError { .. })
            ));
        }

        #[test]
        fn invalid_mode() {
            let sut: Result<Mode, ParseModeError> = "".parse();
            assert_eq!(sut, Err(ParseModeError::InvalidMode(String::new())));
        }
    }

    mod output {
        use super::*;

        #[test]
        fn auto_no_options() {
            let sut: Output = Mode::Auto {
                uidmapping: None,
                gidmapping: None,
                size: None,
            }
            .into();
            assert_eq!(sut, Output::QuadletOptions(String::from("RemapUsers=auto")));
        }

        #[allow(clippy::similar_names)]
        #[test]
        fn auto_all_options() {
            let uidmapping = "100";
            let gidmapping = "200";
            let size = 300;
            let sut: Output = Mode::Auto {
                uidmapping: Some(String::from(uidmapping)),
                gidmapping: Some(String::from(gidmapping)),
                size: Some(size),
            }
            .into();

            assert_eq!(
                sut,
                Output::QuadletOptions(format!(
                    "RemapUsers=auto\n\
                    RemapUid={uidmapping}\n\
                    RemapGid={gidmapping}\n\
                    RemapUidSize={size}"
                ))
            );
        }

        #[test]
        fn container() {
            let id = "name";
            let sut: Output = Mode::Container {
                id: String::from(id),
            }
            .into();
            assert_eq!(sut, Output::PodmanArg(format!("container:{id}")));
        }

        #[test]
        fn host() {
            let sut: Output = Mode::Host.into();
            assert_eq!(sut, Output::PodmanArg(String::from("host")));
        }

        #[test]
        fn keep_id_no_options() {
            let sut: Output = Mode::KeepId {
                uid: None,
                gid: None,
            }
            .into();
            assert_eq!(
                sut,
                Output::QuadletOptions(String::from("RemapUsers=keep-id"))
            );
        }

        #[allow(clippy::similar_names)]
        #[test]
        fn keep_id_all_options() {
            let uid = 100;
            let gid = 200;
            let sut: Output = Mode::KeepId {
                uid: Some(uid),
                gid: Some(gid),
            }
            .into();
            assert_eq!(
                sut,
                Output::PodmanArg(format!("keep-id:uid={uid},gid={gid}"))
            );
        }

        #[test]
        fn nomap() {
            let sut: Output = Mode::Nomap.into();
            assert_eq!(sut, Output::PodmanArg(String::from("nomap")));
        }

        #[test]
        fn ns() {
            let namespace = "namespace";
            let sut: Output = Mode::Ns {
                namespace: String::from(namespace),
            }
            .into();
            assert_eq!(sut, Output::PodmanArg(format!("ns:{namespace}")));
        }
    }
}
