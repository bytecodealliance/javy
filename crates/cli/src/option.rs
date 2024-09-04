use anyhow::{bail, Result};
use std::path::PathBuf;

/// Generic option attributes.
#[derive(Debug)]
pub struct OptionMeta {
    pub name: String,
    pub doc: String,
    pub help: String,
}

pub fn fmt_help(cmd: &str, short: &str, meta: &[OptionMeta]) {
    println!("Available options for {}", cmd);
    for opt in meta {
        print!("\n");
        print!("-{:<3}", short);
        print!("{:>3}", opt.name);
        print!("{} \n", opt.help);
        for line in opt.doc.split("\n") {
            print!("{}", line);
        }
        print!("\n");
    }
}

/// Commonalities between all option groups.
// Until we make more extensive use of this trait.
#[allow(dead_code)]
pub trait OptionGroup {
    fn options() -> Vec<OptionMeta>;
}

#[macro_export]
macro_rules! option_group {
    (
        $(#[$attr:meta])*
        pub enum $opts:ident {
            $(
                $(#[doc = $doc:tt])*
                $opt:ident($type:ty),
            )+
        }

    ) => {

        $(#[$attr])*
        pub enum $opts {
            $(
                $opt($type),
            )+
        }

        impl $crate::option::OptionGroup for $opts {
            fn options() -> Vec<$crate::option::OptionMeta> {
                let mut options = vec![];
                $(
                    let name = stringify!($opt);
                    let kebab_case = name
                    .chars()
                    .flat_map(|c| {
                        if c.is_uppercase() {
                            vec!['-', c.to_ascii_lowercase()]
                        } else {
                            vec![c]
                        }
                    })
                    .collect::<String>();

                    let kebab_case = if kebab_case.starts_with('-') {
                        &kebab_case[1..]
                    } else {
                        &kebab_case
                    };
                    options.push($crate::option::OptionMeta {
                        name: kebab_case.into(),
                        doc: concat!($($doc, "\n",)*).into(),
                        help: <$type>::help().to_string(),
                    });
                )+

                options
            }
        }
    };
}

/// Represents all values that can be parsed.
pub trait OptionValue {
    fn help() -> &'static str;
    fn parse(val: Option<&str>) -> Result<Self>
    where
        Self: Sized;
}

impl OptionValue for bool {
    fn help() -> &'static str {
        "[=y|n]"
    }

    fn parse(val: Option<&str>) -> Result<Self>
    where
        Self: Sized,
    {
        match val {
            None => Ok(true),
            Some("yes") | Some("y") => Ok(true),
            Some("no") | Some("n") => Ok(false),
            Some(_) => bail!("Unknown boolean flag. Valid options: y[es], n[o], (empty)"),
        }
    }
}

impl OptionValue for String {
    fn help() -> &'static str {
        "=val"
    }

    fn parse(val: Option<&str>) -> Result<Self>
    where
        Self: Sized,
    {
        match val {
            Some(v) => Ok(v.into()),
            None => bail!("Expected string argument"),
        }
    }
}

impl OptionValue for PathBuf {
    fn help() -> &'static str {
        "=path"
    }

    fn parse(val: Option<&str>) -> Result<Self>
    where
        Self: Sized,
    {
        match val {
            Some(v) => Ok(PathBuf::from(v)),
            None => bail!("Expected path argument"),
        }
    }
}
