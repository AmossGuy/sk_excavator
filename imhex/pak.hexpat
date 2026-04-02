import std.mem;

#pragma endian little

struct DataEntry {
    u64 file_size; // the size of the file in bytes
    u64 unknown_08;
    u64 unknown_10;
    u64 unknown_18;
    std::mem::Bytes<file_size> file_content;
};

struct DataPointer {
    DataEntry *data_pointer : u64 [[inline]];
};

struct NamePointer {
    char *name_pointer[] : u64 [[inline]];
};

struct Header {
    u32 unknown_00; // set to zero in all the files I've looked at
    u32 file_count; // the numbers of files in the archive
    
    // pointer to a pointer table to the file data entries
    DataPointer *data_array_pointer[file_count] : u64;
    // pointer to a pointer table to the filenames
    NamePointer *name_array_pointer[file_count] : u64;
};

Header header @ 0;

// The following results in a runtime error.
// It's possible that's only because of an ImHex bug:
// https://github.com/WerWolv/ImHex-Patterns/issues/429

/*
#ifdef __IMHEX__
for (u32 i = 0, i < header.file_count, i += 1) {
    str name = header.name_array_pointer[i].name_pointer;
    DataEntry data = header.data_array_pointer[i].data_pointer;
    hex::core::add_virtual_file(name, data.file_content);
}
#endif
*/