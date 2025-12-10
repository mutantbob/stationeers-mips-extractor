/// illustrate a problem with parsing of hex entities.
/// I am not certain if the problem is with Stationeers or the rxml crate.
#[test]
fn test1() -> Result<(), rxml::Error> {
    let reader = "<a>banana&#xb;orange</a>";

    let mut r2 = rxml::Reader::new(reader.as_bytes());

    {
        let x = r2.read()?.unwrap();
        if let rxml::Event::StartElement(m, qname, map) = x {
            assert_eq!("a", qname.1.as_str());
        } else {
            assert!(false)
        }
    }
    {
        let evt = r2.read()?.unwrap();
        if let rxml::Event::Text(m, payload) = &evt {
            assert_eq!("banana", payload.as_str());
        } else {
            assert!(false)
        }
        println!("{:#?}", evt);
    }
    {
        let evt = r2.read();
        println!("{:#?}", &evt);
        assert!(evt.is_ok());
    }
    Ok(())
}
