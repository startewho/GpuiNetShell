//! `Image`: a bitmap or SVG loaded from an asset path, with an object-fit mode.
//!
//! Sources are resolved through the application [`AssetSource`], which serves
//! the bundled icons and falls back to the filesystem for plain paths. gpui's
//! asset cache owns the decoded texture and releases it once no view references
//! it, so an image removed from the tree frees its decoded data.
//!
//! [`AssetSource`]: gpui::AssetSource

use std::sync::Arc;

use gpui::{
    img, AnyElement, IntoElement as _, ObjectFit, Refineable as _, Styled as _, StyledImage as _,
};

use crate::registry::{
    ArgumentDescriptor, ArgumentSchema, ComponentArgument, ComponentDescriptor,
    ComponentMaterializer, ComponentPayload, ComponentRegistry, ConstructorDescriptor,
    MaterializeRequest, MethodDescriptor,
};

#[derive(Clone)]
struct ImagePayload(String);

#[derive(Clone, Copy)]
enum ImageOp {
    Fit(Fit),
}

#[derive(Clone, Copy)]
enum Fit {
    Cover,
    Contain,
    Fill,
    None,
    ScaleDown,
}

impl Fit {
    fn object_fit(self) -> ObjectFit {
        match self {
            Fit::Cover => ObjectFit::Cover,
            Fit::Contain => ObjectFit::Contain,
            Fit::Fill => ObjectFit::Fill,
            Fit::None => ObjectFit::None,
            Fit::ScaleDown => ObjectFit::ScaleDown,
        }
    }
}

struct ImageMaterializer;

impl ComponentMaterializer for ImageMaterializer {
    fn materialize(&self, mut request: MaterializeRequest<'_>) -> Result<AnyElement, String> {
        let path = request
            .payload()
            .downcast_ref::<ImagePayload>()
            .ok_or_else(|| "Image received an incompatible payload".to_string())?
            .0
            .clone();
        if request.children_len() != 0 {
            return Err("Image does not accept children".to_string());
        }
        let mut image = img(path);
        for op in request
            .methods()
            .filter_map(|method| method.payload().downcast_ref::<ImageOp>())
        {
            image = match op {
                ImageOp::Fit(value) => image.object_fit(value.object_fit()),
            };
        }
        image.style().refine(&request.take_style());
        Ok(image.into_any_element())
    }
}

pub(super) fn register(registry: &mut ComponentRegistry) {
    registry
        .register(
            ComponentDescriptor::new("Image", Arc::new(ImageMaterializer))
                .with_constructors(vec![ConstructorDescriptor::new(
                    "Image",
                    vec![ArgumentDescriptor::new("source", ArgumentSchema::String)],
                    |arguments| match arguments {
                        [ComponentArgument::String(source)] if !source.trim().is_empty() => {
                            Ok(ComponentPayload::new(ImagePayload(source.clone())))
                        }
                        _ => Err("Image expects a non-empty source path".into()),
                    },
                )])
                .with_methods(vec![MethodDescriptor::new(
                    "fit",
                    vec![ArgumentDescriptor::new(
                        "fit",
                        ArgumentSchema::Enum(&["cover", "contain", "fill", "none", "scale_down"]),
                    )],
                    |arguments| match arguments {
                        [ComponentArgument::Enum(value)] => match value.as_str() {
                            "cover" => Ok(ComponentPayload::new(ImageOp::Fit(Fit::Cover))),
                            "contain" => Ok(ComponentPayload::new(ImageOp::Fit(Fit::Contain))),
                            "fill" => Ok(ComponentPayload::new(ImageOp::Fit(Fit::Fill))),
                            "none" => Ok(ComponentPayload::new(ImageOp::Fit(Fit::None))),
                            "scale_down" => Ok(ComponentPayload::new(ImageOp::Fit(Fit::ScaleDown))),
                            _ => Err(format!("unsupported Image fit `{value}`")),
                        },
                        _ => Err("Image.fit expects a fit literal".into()),
                    },
                )
                .with_documentation("Sets the object-fit mode.")])
                .with_documentation(
                    "An image or SVG loaded from the asset source by path; size and radius \
                     come from style.",
                ),
        )
        .expect("the built-in Image descriptor is valid");
}
