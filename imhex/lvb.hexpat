import std.mem;

#pragma endian little

struct Row5Entry {
    u32 unknown[2];
};

struct Row4Entry {
    u32 unknown[6];
};

struct Row3Entry {
    u32 unknown[5];
};

struct Row2Entry {
    u32 unknown[12];
};

struct Row1Entry {
    u32 unknown[2];
};

struct Path {
    u32 unknown_00;
    if (std::mem::read_unsigned($, 1) != 0) {
        char name[32];
        u32 unknown_arr[26];
    }
};

struct PathPointer {
    Path *path_pointer : u64;
};

struct Header {
    u32 row1_extra;
    u32 row1_count;
    Row1Entry *row1_pointer[row1_count] : u64;
    
    u32 row2_extra;
    u32 row2_count;
    // what could this possibly mean?
    Row2Entry *row2_pointer[row2_count + row2_extra] : u64;
    
    u32 row3_extra;
    u32 row3_count;
    Row3Entry *row3_pointer[row3_count] : u64;
    
    u32 row4_extra;
    u32 row4_count;
    Row4Entry *row4_pointer[row4_count] : u64;
    
    u32 row5_extra;
    u32 row5_count;
    Row5Entry *row5_pointer[row5_count] : u64;
    
    u32 row6_extra;
    u32 row6_count;
    PathPointer *row6_pointer[row6_count] : u64;
    
    u32 string_table_extra;
    u32 string_table_size;
    std::mem::Bytes<string_table_size> *string_table_pointer : u64;
};

Header header @ 0;