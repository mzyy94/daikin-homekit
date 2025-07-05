macro_rules! set_child_prop {
    ( $p:tt . $name:ident = $propval:tt) => {
        match $p {
            crate::property::Property::Tree{ref mut children, ..} => {
                let found = children.iter_mut().find(|p| match p {
                    crate::property::Property::Tree { name, .. } => name == stringify!($name),
                    crate::property::Property::Node(crate::property::Item { name, .. }) => name == stringify!($name),
                });
                if let Some(crate::property::Property::Node(crate::property::Item { ref mut value, .. })) = found {
                    *value = $propval;
                } else {
                    let pp = crate::property::Property::new(stringify!($name), $propval);
                    children.push(pp);
                }

            }
            _ => unreachable!(),
        }
    };
    ({ $p:expr } . $name:ident $($rest:tt)*) => {
        set_child_prop!(
            {
                match $p {
                    crate::property::Property::Tree{ref mut children, ..} => {
                        let found = children.iter_mut().find(|p| match p {
                            crate::property::Property::Tree { name, .. } => name == stringify!($name),
                            crate::property::Property::Node(crate::property::Item { name, .. }) => name == stringify!($name),
                        });
                        if let Some(p) = found {
                            p
                        } else {
                            let pp = crate::property::Property::new_tree(stringify!($name));
                            children.push(pp);
                            children.iter_mut().last().unwrap()
                        }
                    },
                    _ => unreachable!(),
                }
            } $($rest)*
        )
    };
}

macro_rules! set_prop {
    (&mut $root:tt . $path:literal $($rest:tt)*) => {
        set_child_prop!(
            {
                &mut match $root.requests.iter_mut().find(|r| r.to == $path) {
                    Some(ref mut r) => r,
                    None => {
                        let req = crate::request::Request {
                            op: 3,
                            pc: crate::property::Property::new_tree("dgc_status".into()),
                            to: $path.into(),
                        };
                        $root.requests.push(req);
                        $root.requests.last_mut().unwrap()
                    }
                }.pc
            }  $($rest)*
        )
    };
}

macro_rules! get_child_prop {
    ({ $popt:expr }) => {
        $popt.ok_or(crate::error::Error::NoProperty)
    };
    ({ $popt:expr } -> Meta) => {{
        let Some(crate::property::Property::Node( item )) = $popt else {
            panic!("Expected a Property::Node, but got something else.");
            // return crate::status::Meta {step: 0.0, min: 0.0, max: 0.0, digits: 0};
        };
        let (step, min, max) = item.meta();
        let digits = item.size();
        crate::status::Meta {step, min, max, digits}
    }
    };
    ({ $popt:expr } as $ty:ty) => {{
        let Some(crate::property::Property::Node( item )) = $popt else {
            panic!("Expected a Property::Node, but got something else.");
        };
        item.get_f32().map(|v| v as $ty)
    }};
    ({ $popt:expr } .into()) => {{
        let Some(crate::property::Property::Node( item )) = $popt else {
            panic!("Expected a Property::Node, but got something else.");
        };
        serde_json::from_value(item.get_f32().map(|v| serde_json::Value::Number(serde_json::Number::from(v as u8))).unwrap_or_default()).unwrap_or_default()
    }};
    ({ $popt:expr } .to_string()) => {{
        let Some(crate::property::Property::Node( item )) = $popt else {
            panic!("Expected a Property::Node, but got something else.");
        };
        item.get_string()
    }};
    ({ $popt:expr } . $name:ident $($rest:tt)*) => {
        get_child_prop!(
            { $popt.and_then(|p| p.find(stringify!($name))) } $($rest)*
        )
    };
}

macro_rules! get_prop {
    ($root:tt . $path:literal $($rest:tt)*) => {
        get_child_prop!(
            { $root.responses.iter().find(|&r| r.from == $path).and_then(|r| r.content.as_ref()) } $($rest)*
        )
    };
}

macro_rules! propvalue {
    ($dkst:tt . $name:ident) => {
        $dkst.$name.map(|v| {
            crate::property::PropValue::from(
                v as f32,
                $dkst.meta.$name.step,
                $dkst.meta.$name.digits,
            )
        })
    };
    ($dkst:tt . $name:ident as $ty:ty) => {
        $dkst.$name.map(|v| {
            crate::property::PropValue::from(
                v as $ty as f32,
                $dkst.meta.$name.step,
                $dkst.meta.$name.digits,
            )
        })
    };
}

#[cfg(test)]
mod tests {
    use crate::property::PropValue;
    use crate::request::DaikinRequest;
    use crate::response::DaikinResponse;
    use crate::status::DaikinStatus;

    #[test]
    fn get_prop() {
        let res: DaikinResponse = serde_json::from_str(include_str!("./fixtures/status.json"))
            .expect("Invalid JSON file.");

        let p = get_prop!(res."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A001.p_03);
        assert_eq!(
            format!("{:?}", p),
            r#"Ok(crate::property::Item { name: "p_03", value: String("3800"), metadata: Binary(Step(BinaryStep { step: 241, min: "0000", max: "FF00" })) })"#
        );

        let p = get_prop!(res."/hoge".fuga.piyo);
        assert_eq!(format!("{:?}", p), r#"Err(NoProperty)"#);
    }

    #[test]
    fn set_prop() {
        let mut req = DaikinRequest { requests: vec![] };

        let pv = PropValue::String("3800".into());
        set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A001.p_03 = pv);

        assert_eq!(
            serde_json::to_string(&req).unwrap(),
            r#"{"requests":[{"op":3,"pc":{"pn":"dgc_status","pch":[{"pn":"e_1002","pch":[{"pn":"e_A001","pch":[{"pn":"p_03","pv":"3800"}]}]}]},"to":"/dsiot/edge/adr_0100.dgc_status"}]}"#
        );
    }

    #[test]
    fn propvalue() {
        let res: DaikinResponse = serde_json::from_str(include_str!("./fixtures/status.json"))
            .expect("Invalid JSON file.");
        let status: DaikinStatus = res.into();

        let pv = propvalue!(status.target_heating_temperature);
        assert_eq!(format!("{:?}", pv), r#"Some(String("32"))"#);

        let pv = propvalue!(status.mode as u8);
        assert_eq!(format!("{:?}", pv), r#"Some(String("0200"))"#);
    }
}
