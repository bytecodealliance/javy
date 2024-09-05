use anyhow::{bail, Result};
use std::path::PathBuf;

/// An option group used for parsing strings to their group option representation.
#[derive(Clone, Debug)]
pub struct GroupOption<T>(pub Vec<T>);

#[derive(Clone)]
pub struct GroupOptionParser<T>(pub std::marker::PhantomData<T>);

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
        println!();
        print!("-{:<3}", short);
        print!("{:>3}", opt.name);
        println!("{}", opt.help);
        for line in opt.doc.split('\n') {
            print!("{}", line);
        }
        println!();
    }
}

/// Commonalities between all option groups.
// Until we make more extensive use of this trait.
#[allow(dead_code)]
pub trait GroupDescriptor {
    fn options() -> Vec<OptionMeta>;
}

pub trait GroupOptionBuilder: Clone + Sized + Send + Sync + 'static {
    fn parse(val: &str) -> anyhow::Result<Self>
    where
        Self: Sized;
}

pub fn to_kebab_case(val: &str) -> String {
    let mut kebab_case = val
        .chars()
        .flat_map(|c| {
            if c.is_uppercase() {
                vec!['-', c.to_ascii_lowercase()]
            } else {
                vec![c]
            }
        })
        .collect::<String>();

    if kebab_case.starts_with('-') {
        kebab_case.remove(0);
    }

    kebab_case
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

        impl $crate::option::GroupDescriptor for $opts {
            fn options() -> Vec<$crate::option::OptionMeta> {
                let mut options = vec![];
                $(
                    let name = $crate::option::to_kebab_case(stringify!($opt));
                    options.push($crate::option::OptionMeta {
                        name: name.to_string(),
                        doc: concat!($($doc, "\n",)*).into(),
                        help: <$type>::help().to_string(),
                    });
                )+

                options
            }
        }

        impl $crate::option::GroupOptionBuilder for $opts {
            fn parse(val: &str) -> anyhow::Result<Self> {
                let mut parts = val.splitn(2, '=');
                let key = parts.next().ok_or_else(|| anyhow!("Expected key. None found"))?;
                let val = parts.next();

                $(
                    if key == $crate::option::to_kebab_case(stringify!($opt)) {
                        return Ok($opts::$opt($crate::option::OptionValue::parse(val)?));
                    }
                )+

                    Err(anyhow!("Invalid argument"))
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
