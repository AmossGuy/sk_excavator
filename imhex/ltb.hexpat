import std.io;
import std.mem;

#pragma endian little

struct WflzContainer {
    char magic[4];
    
    if (magic == "WFLZ") {
        u32 compressed_size;
        u32 decompressed_size;
        std::mem::Bytes<compressed_size> data;
    }
};

struct WflzContainerPointer {
    WflzContainer *wflz_container_pointer : u64;
};

struct Mystery6 {
    u32 unknown[5];
};

struct Mystery5 {
    u32 unknown[2];
};

struct Mystery2 {
    u32 unknown[20];
};

struct Mystery1 {
    u32 unknown[5];
};

struct Layer {
    char name[32];
    u32 unknown[24];
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
    u32 mystery2_count;
    Mystery2 *mystery2_pointer[mystery2_count] : u64;
    
    u32 unknown_40;
    u32 mystery3_count;
    u32 *mystery3_pointer[mystery3_count] : u64;
    
    u32 unknown_50;
    u32 mystery4_count;
    u16 *mystery4_pointer[mystery4_count] : u64;
    
    u32 unknown_60;
    u32 mystery5_count;
    Mystery5 *mystery5_pointer[mystery5_count] : u64;
    
    u32 unknown_70;
    u32 mystery6_count;
    Mystery6 *mystery6_pointer[mystery6_count] : u64;
    
    u32 unknown_80;
    u32 wflz_container_count;
    WflzContainerPointer *wflz_containers[wflz_container_count] : u64;
};

Header header @ 0;