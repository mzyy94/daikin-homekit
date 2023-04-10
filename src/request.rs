use crate::property::Property;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Request {
    pub op: u8,
    pub pc: Property,
    pub to: String,
}

macro_rules! set_child_prop {
    ( $v:tt . $key:ident = $val:tt) => {
        match $v {
            Property::Tree{ref mut pch, ..} => {
                let value = PropValue::String( $val.to_string());
                let found = pch.iter_mut().find(|p| match p {
                    Property::Tree { pn, .. } => pn == stringify!($key),
                    Property::Item { pn, .. } => pn == stringify!($key),
                });
                if let Some(Property::Item { ref mut pv, .. }) = found {
                    *pv = Some(value);
                } else {
                    let pp = Property::new(stringify!($key), value);
                    pch.push(pp);
                }

            }
            _ => unreachable!(),
        }
    };
    ({ $v:expr } . $key:ident $($rest:tt)*) => {
        set_child_prop!(
            {
                match $v {
                    Property::Tree{ref mut pch, ..} => {
                        let found = pch.iter_mut().find(|p| match p {
                            Property::Tree { pn, .. } => pn == stringify!($key),
                            Property::Item { pn, .. } => pn == stringify!($key),
                        });
                        if let Some(p) = found {
                            p
                        } else {
                            let pp = Property::new_tree(stringify!($key));
                            pch.push(pp);
                            pch.iter_mut().last().unwrap()
                        }
                    },
                    _ => unreachable!(),
                }
            } $($rest)*
        )
    };
}

macro_rules! set_prop {
    (&mut $v:tt . $key:literal $($rest:tt)*) => {
        set_child_prop!(
            {
                &mut match $v.requests.iter_mut().find(|r| r.to == $key) {
                    Some(ref mut r) => r,
                    None => {
                        let req = Request {
                            op: 3,
                            pc: Property::new_tree("dgc_status".into()),
                            to: $key.into(),
                        };
                        $v.requests.push(req);
                        $v.requests.last_mut().unwrap()
                    }
                }.pc
            }  $($rest)*
        )
    };
}
