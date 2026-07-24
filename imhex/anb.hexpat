import std.io;
import std.mem;
import std.ptr;

#pragma endian little
#pragma magic [ 59 43 53 4E ] @ 0

struct DataBlock {
    char magic[4];
    u32 size;
    std::mem::Bytes<size> content;
    
    if (magic != "\xff\xff\xff\x00") {
        std::print("wrong data block magic: {} @ {}", magic, $);
    }
};

struct DataBlockPointer {
    DataBlock *data_block_pointer: u64 [[inline]];
};

using NodePointer;
struct TreeNode {
    u32 type;
    u32 child_count;
    NodePointer *child_pointer[child_count] : u64;
    
    if (type == 0) {
        // no extra data
    } else if (type == 1) {
        u32 sprite_width;
        u32 sprite_height;
        u32 e_unknown_08;
        u32 e_unknown_0C;
        DataBlock *e_pointer : u64;
    } else if (type == 2) {
        u32 e_unknown_00;
        u32 e_unknown_04;
        DataBlock *e_pointer : u64;
    } else if (type == 4) {
        u32 e_unknown_00;
        u32 e_unknown_01;
    } else if (type == 5) {
        u32 e_unknown_00;
        u32 e_unknown_01;
    } else if (type == 7) {
        u32 e_unknown_00;
        u32 e_unknown_04;
        u32 e_unknown_08;
        u32 e_unknown_0C;
        u32 e_unknown_10;
        u32 e_unknown_14;
        u32 e_unknown_18;
        u32 e_unknown_1C;
    } else if (type == 8) {
        u32 e_unknown_00;
        u32 e_unknown_04;
        DataBlock *e_pointer : u64;
    } else if (type == 9) {
        std::ptr::NullablePtr<DataBlock, u64> e_pointer;
     } else if (type == 10) {
        u32 e_unknown_00;
        u32 e_unknown_04;
        u32 e_unknown_08;
        u32 e_unknown_0C;
    } else if (type == 11) {
        u32 e_unknown_00;
        u32 e_unknown_04;
    } else if (type == 12) {
        u32 e_unknown_00;
        u32 e_unknown_04;
    } else {
        std::print("unknown node type {} @ {:X}", type, $);
    }
};

struct NodePointer {
    TreeNode *pointer : u64 [[inline]];
};

struct Header {
    char magic[4];
    if (magic != "YCSN") {
        std::print("wrong file magic: {}", magic);
    }
    u32 unknown_04;
    u32 unknown_08;
    u32 unknown_0C;
    u32 unknown_10;
    u32 unknown_14;
    TreeNode root_node;
};

Header header @ 0;
