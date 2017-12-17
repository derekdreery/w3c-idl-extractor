#[macro_use] extern crate html5ever;
#[macro_use] extern crate failure_derive;
extern crate failure;

use std::io;
use std::default::Default;
use std::string::String;
use std::ascii::AsciiExt;

use html5ever::parse_document;
use html5ever::rcdom::{NodeData, RcDom, Handle};
use html5ever::tendril::TendrilSink;


#[derive(Fail, Debug)]
pub enum ExtractorError {
    #[fail(display = "i/o error occured during reading input")]
    InputIo(#[cause] io::Error)
}

/// Check if a node is an element with name `pre` and class `idl`
fn is_pre_class_idl(node: &NodeData) -> bool {
    if let NodeData::Element { ref name, ref attrs, .. } = *node {
        if name.local.as_ref().eq_ignore_ascii_case("pre") {
            let mut has_class_idl = false;
            for attr in attrs.borrow().iter() {
                let name = &attr.name;
                if name.ns == ns!()
                    && name.local.as_ref() == "class"
                    && attr.value.contains("idl")
                {
                    has_class_idl = true;
                }
            }
            has_class_idl
        } else {
            false
        }
    } else {
        false
    }
}

/// Get all text content within an element
fn get_text_content(node: Handle, acc: &mut String) {
    use std::fmt::Write;

    if let NodeData::Text { ref contents } = node.data {
        write!(acc, "{}", contents.borrow()).unwrap() // cannot fail to string
    } else {
        for child in node.children.borrow().iter() {
            get_text_content(child.clone(), acc);
        }
    }
}

/// Recursively search for IDL fragments
fn search_for_idl(node: Handle, mut acc: &mut Vec<String>) {
    if is_pre_class_idl(&node.data) {
        let mut idl = String::new();
        get_text_content(node, &mut idl);
        acc.push(idl);
    } else {
        for child in node.children.borrow().iter() {
            search_for_idl(child.clone(), &mut acc);
        }
    }
}

/// Return a `Vec` of idl fragments from the document
///
/// # todo
///
/// There are some things that could be tried to see if they improve the output.
///
///  1. Use a streaming parser rather than loading all the output into memory (I don't know how to
///     use html5ever as a streaming parser)
///  2. An alternative strategy for searching for IDL:
///     (I'm not sure if this is better or worse than just testing all `<pre class=".. idl ..">`)
///     - for every node in the tree, if all text from it and its children form valid IDL,
///       - then return an IDL fragment
///       - else recurse into the children
pub fn extract_idl<R>(mut reader: R) -> Result<Vec<String>, ExtractorError>
    where R: io::Read
{
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut reader)
        .map_err(ExtractorError::InputIo)?;

    let mut idl_list = Vec::new();
    search_for_idl(dom.document, &mut idl_list);
    if !dom.errors.is_empty() {
        println!("\nParse errors:");
        for err in dom.errors.into_iter() {
            println!("    {}", err);
        }
    }
    Ok(idl_list)
}

