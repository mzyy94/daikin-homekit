macro_rules! set_child_prop {
    ( $p:tt . $name:ident = $item:expr) => {
        match $p {
            crate::property::Property::Tree{ref mut children, ..} => {
                let found = children.iter_mut().find(|p| match p {
                    crate::property::Property::Tree { name, .. } => name == stringify!($name),
                    crate::property::Property::Node(crate::property::Item { name, .. }) => name == stringify!($name),
                });
                if let Some(crate::property::Property::Node(crate::property::Item { ref mut value, .. })) = found {
                    *value = $item.value;
                } else {
                    let pp = crate::property::Property::Node(crate::property::Item {
                        name: $item.name.to_string(),
                        value: $item.value,
                        metadata: crate::property::Metadata::Undefined,
                        phantom: std::marker::PhantomData,
                    });
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
        match $popt {
            Some(crate::property::Property::Node( item )) => {
                crate::property::Item {
                    name: item.name.clone(),
                    value: item.value.clone(),
                    metadata: item.metadata.clone(),
                    phantom: std::marker::PhantomData,
                }
            },
            _ => crate::property::Item {
                name: String::new(),
                value: crate::property::PropValue::Null,
                metadata: crate::property::Metadata::Undefined,
                phantom: std::marker::PhantomData,
            }
        }
    };
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

#[cfg(test)]
mod tests {
    use crate::property::{Item, PropValue};
    use crate::request::DaikinRequest;
    use crate::response::DaikinResponse;

    #[test]
    fn get_prop() {
        let res: DaikinResponse = serde_json::from_str(include_str!("./fixtures/status.json"))
            .expect("Invalid JSON file.");

        let p: Item = get_prop!(res."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A001.p_03);
        assert_eq!(
            format!("{:?}", p),
            r#"Item { name: "p_03", value: String("3800"), metadata: Binary(Step(BinaryStep { range: 0.0..=25.5, step: 0.1 })), phantom: PhantomData<fn() -> f32> }"#
        );

        let p: Item = get_prop!(res."/hoge".fuga.piyo);
        assert_eq!(
            format!("{:?}", p),
            r#"Item { name: "", value: Null, metadata: Undefined, phantom: PhantomData<fn() -> f32> }"#
        );
    }

    #[test]
    fn set_prop() {
        let mut req = DaikinRequest { requests: vec![] };

        let item: Item<f32> = Item {
            name: "p_03".into(),
            value: PropValue::String("3800".into()),
            metadata: crate::property::Metadata::Undefined,
            phantom: std::marker::PhantomData,
        };
        set_prop!(&mut req."/dsiot/edge/adr_0100.dgc_status".e_1002.e_A001.p_03 = item);

        assert_eq!(
            serde_json::to_string(&req).unwrap(),
            r#"{"requests":[{"op":3,"pc":{"pn":"dgc_status","pch":[{"pn":"e_1002","pch":[{"pn":"e_A001","pch":[{"pn":"p_03","pv":"3800"}]}]}]},"to":"/dsiot/edge/adr_0100.dgc_status"}]}"#
        );
    }
}
