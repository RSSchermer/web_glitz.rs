struct Texture2D {

}

struct Texture2DArray {
    layers: Vec<TextureLayer>
}

struct TextureCube {
    positive_x_layer: TextureLayer,
    negative_x_layer: TextureLayer,
    positive_y_layer: TextureLayer,
    negative_y_layer: TextureLayer,
    positive_z_layer: TextureLayer,
    negative_z_layer: TextureLayer,
}

struct TextureLayer {

}

struct MipMap {
    levels: Vec<MipMapLevel>
}

struct TextureImage {

}

enum ImageSource<'a> {
    Blank(u32, u32),
    Bytes(&'a [u8], u32, u32),
    ImageElement(&'a ImageElement),
    ImageElementRegion(),
    CanvasElement(&'a CanvasElement),
    CanvasElementRegion()
}