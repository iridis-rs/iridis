use flarrow_message::prelude::{
    thirdparty::{arrow_array::*, arrow_data::*, arrow_schema::*, *},
    *,
};

#[derive(Debug)]
struct Metadata {
    name: Option<String>,
    width: u32,
    height: u32,
}

impl ArrowMessage for Metadata {
    fn field(name: impl Into<String>) -> Field {
        make_union_fields(
            name,
            vec![
                Option::<String>::field("name"),
                Option::<u32>::field("width"),
                Option::<u32>::field("height"),
            ],
        )
    }

    fn try_from_arrow(data: ArrayData) -> Result<Self>
    where
        Self: Sized,
    {
        let (map, children) = unpack_union(data);

        Ok(Metadata {
            name: extract_union_data("name", &map, &children)?,
            width: extract_union_data("width", &map, &children)?,
            height: extract_union_data("height", &map, &children)?,
        })
    }

    fn try_into_arrow(self) -> Result<ArrayRef> {
        let union_fields = get_union_fields::<Self>()?;

        make_union_array(
            union_fields,
            vec![
                self.name.try_into_arrow()?,
                self.width.try_into_arrow()?,
                self.height.try_into_arrow()?,
            ],
        )
    }
}

impl TryFrom<ArrayData> for Metadata {
    type Error = eyre::Report;

    fn try_from(data: ArrayData) -> Result<Self> {
        Metadata::try_from_arrow(data)
    }
}

impl TryFrom<Metadata> for ArrayData {
    type Error = eyre::Report;

    fn try_from(metadata: Metadata) -> Result<Self> {
        metadata.try_into_arrow().map(|array| array.into_data())
    }
}

#[derive(Debug)]
struct Image {
    data: UInt8Array,
    metadata: Option<Metadata>,
}

impl ArrowMessage for Image {
    fn field(name: impl Into<String>) -> Field {
        make_union_fields(
            name,
            vec![
                UInt8Array::field("data"),
                Option::<Metadata>::field("metadata"),
            ],
        )
    }

    fn try_from_arrow(data: ArrayData) -> Result<Self>
    where
        Self: Sized,
    {
        let (map, children) = unpack_union(data);

        Ok(Image {
            data: extract_union_data("data", &map, &children)?,
            metadata: extract_union_data("metadata", &map, &children)?,
        })
    }

    fn try_into_arrow(self) -> Result<ArrayRef> {
        let union_fields = get_union_fields::<Self>()?;

        make_union_array(
            union_fields,
            vec![self.data.try_into_arrow()?, self.metadata.try_into_arrow()?],
        )
    }
}

impl TryFrom<ArrayData> for Image {
    type Error = eyre::Report;

    fn try_from(data: ArrayData) -> Result<Self> {
        Image::try_from_arrow(data)
    }
}

impl TryFrom<Image> for ArrayData {
    type Error = eyre::Report;

    fn try_from(image: Image) -> Result<Self> {
        image.try_into_arrow().map(|array| array.into_data())
    }
}

fn main() -> Result<()> {
    let image = Image {
        data: UInt8Array::from(vec![1, 2, 3]),
        metadata: Some(Metadata {
            name: Some("example".to_string()),
            width: 12,
            height: 12,
        }),
    };

    println!("{:?}", image);

    let arrow = ArrayData::try_from(image)?;
    let image = Image::try_from(arrow)?;

    println!("{:?}", image);

    Ok(())
}
