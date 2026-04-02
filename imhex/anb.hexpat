import std.io;
import std.mem;

#pragma endian little
#pragma magic [ 59 43 53 4E ] @ 0

struct DataBlock {
    char magic[4];
    u32 size;
    std::mem::Bytes<size> content;
    
    if (magic != "\xff\xff\xff\x00") {
        std::print("wrong data block magic: {}", magic);
    }
};

struct DataBlockPointer {
    DataBlock *data_block_pointer: u64 [[inline]];
};

struct SubSubInfoEntry {
    u32 unknown_00;
    u32 unknown_04;
};

struct SubSubInfoPointer {
    SubSubInfoEntry *sub_sub_info_pointer : u64 [[inline]];
};

struct SubInfoEntry {
    u32 unknown_00;
    u32 sub_sub_count;
    SubSubInfoPointer *i_pointer[sub_sub_count] : u64;
    DataBlockPointer d_pointer[sub_sub_count];
};

struct SubInfoPointer {
    SubInfoEntry *sub_info_pointer : u64 [[inline]];
};

struct InfoEntry {
    u32 type;
    u32 sub_count;
    SubInfoPointer *pointer[sub_count] : u64;
    
    std::print(type);
    if (type == 1) {
        u32 e_unknown_00;
        u32 e_unknown_04;
        u32 e_unknown_08;
        u32 e_unknown_0C;
        DataBlock *e_pointer : u64;
   }
};

struct InfoPointer {
    InfoEntry *info_pointer : u64 [[inline]];
};

struct InfoHeaderExtend {
    
};

struct InfoHeader {
    u32 unknown_00;
    u32 count;
    InfoPointer *pointer[count] : u64;
    
    
};

struct InfoHeaderPointer {
    InfoHeader *info_header_pointer : u64 [[inline]];
};

struct OuterThing {
    u32 unknown_00;
    u32 unknown_04;
    InfoHeaderPointer *pointer[unknown_04] : u64;
    
    // hopefully right
    u32 unknown_10;
    u32 unknown_14;
    u32 unknown_18;
    u32 unknown_1C;
    DataBlock *pointer2 : u64;
};

struct OuterThingPointer {
    OuterThing *outer_thing_pointer : u64 [[inline]];
};

struct Header {
    char magic[4];
    u32 unknown_04;
    u32 unknown_08;
    u32 unknown_0C;
    u32 unknown_10;
    u32 unknown_14;
    u32 unknown_18;
    u32 unknown_1C;
    OuterThingPointer *pointer[1]: u64;
    
    if (magic != "YCSN") {
        std::print("wrong file magic: {}", magic);
    }
};

Header header @ 0;