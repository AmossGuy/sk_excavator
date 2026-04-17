import std.core;
import std.io;
import std.mem;

#pragma endian little

// note: despite my use of the term "image" here, this structure also stores palettes
struct ImagePointer {
    u32 size = parent.image_meta_pointer[std::core::array_index()].data_size;
    std::mem::Bytes<size> *image_pointer : u64 [[inline]];
};

struct Mystery6 {
    u32 unknown[5];
};

struct ImageMetadata {
    u32 unknown_00;
    u32 is_compressed;
    u32 image_width;
    u32 image_height;
    u32 unknown_more[14];
    u32 data_size;
};

struct Mystery1 {
    u32 unknown[5];
};

struct Layer {
    char name[32];
    u32 unknown_arr1[14];
    u32 unknown_38;
    u32 chunkmap_width;
    u32 chunkmap_height;
    u32 chunkmap_offset;
    u32 unknown_arr2[6];
};

struct Header {
    u32 unknown_00;
    u32 unknown_04;
    u32 unknown_08;
    u32 unknown_0C;
    
    u32 unknown_10;
    u32 layer_count;
    Layer *layer_pointer[layer_count] : u64;
    
    u32 unknown_20;
    u32 mystery1_count;
    Mystery1 *mystery1_pointer[mystery1_count] : u64;
    
    u32 unknown_30;
    u32 image_meta_count;
    ImageMetadata *image_meta_pointer[image_meta_count] : u64;
    
    u32 unknown_40;
    u32 chunkmap_size;
    std::mem::Bytes<chunkmap_size * 4> *chunkmap_pointer : u64;
    
    u32 unknown_50;
    u32 tilemap_size;
    std::mem::Bytes<tilemap_size * 2> *tilemap_pointer : u64;
    
    u32 unknown_60;
    u32 tileset_size;
    std::mem::Bytes<tileset_size * 8> *tileset_pointer : u64;
    
    u32 unknown_70;
    u32 mystery6_count;
    Mystery6 *mystery6_pointer[mystery6_count] : u64;
    
    u32 unknown_80;
    u32 image_count;
    ImagePointer *images_pointer[image_count] : u64;
};

Header header @ 0;