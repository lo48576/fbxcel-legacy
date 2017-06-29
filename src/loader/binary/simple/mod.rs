//! Simple binary loader.

use parser::binary::{Parser, ParserSource, Event};
pub use self::error::{Result, Error};
pub use self::generic::{GenericNode, OwnedAttribute};

pub mod error;
pub mod generic;
pub mod fbx7400;


/// FBX loader.
#[derive(Debug)]
struct FbxLoaderInner<R, P> {
    /// FBX version.
    version: u32,
    /// Parser.
    parser: P,
    /// Dummy.
    _r: ::std::marker::PhantomData<R>,
}


/// FBX 7.4 compatible loader.
#[derive(Debug)]
pub struct FbxLoader7400<R, P> {
    /// Inner loader.
    inner: FbxLoaderInner<R, P>,
}

impl<R: ParserSource, P: Parser<R>> FbxLoader7400<R, P> {
    /// Creates a new `FbxLoader7400` from the given inner loader data.
    fn new(inner: FbxLoaderInner<R, P>) -> Self {
        FbxLoader7400 { inner: inner }
    }

    /// Load FBX 7.4 compatible data.
    pub fn load<O>(self, objs_loader: O) -> Result<fbx7400::Fbx7400<O>>
    where
        O: fbx7400::LoadObjects7400,
    {
        fbx7400::Fbx7400::load_from_parser(self.inner.version, self.inner.parser, objs_loader)
    }
}


/// FBX loader.
#[derive(Debug)]
pub enum FbxLoader<R, P> {
    /// FBX 7.4 compatible.
    Fbx7400(FbxLoader7400<R, P>),
}

impl<R: ParserSource, P: Parser<R>> FbxLoader<R, P> {
    /// Loads FBX structure from the given parser.
    ///
    /// # Panics
    /// Panics if the parser has already emitted some event (i.e. if the given parser didn't return
    /// the `StartFbx` first).
    pub fn load_from_parser(mut parser: P) -> Result<Self> {
        let version = match parser.next_event()? {
            Event::StartFbx(header) => header.version,
            ev => {
                panic!(
                    "FBX binary parser should return `StartFbx` as the first event but got \
                        `{:?}`",
                    ev
                )
            },
        };
        let inner = FbxLoaderInner {
            version: version,
            parser: parser,
            _r: Default::default(),
        };
        match version {
            7400...7599 => Ok(FbxLoader::Fbx7400(FbxLoader7400::new(inner))),
            _ => {
                error!("Unsupported FBX version: {}", version);
                unimplemented!()
            },
        }
    }
}
