# CMAKE generated file: DO NOT EDIT!
# Generated by "Unix Makefiles" Generator, CMake Version 3.20

# Delete rule output on recipe failure.
.DELETE_ON_ERROR:

#=============================================================================
# Special targets provided by cmake.

# Disable implicit rules so canonical targets will work.
.SUFFIXES:

# Disable VCS-based implicit rules.
% : %,v

# Disable VCS-based implicit rules.
% : RCS/%

# Disable VCS-based implicit rules.
% : RCS/%,v

# Disable VCS-based implicit rules.
% : SCCS/s.%

# Disable VCS-based implicit rules.
% : s.%

.SUFFIXES: .hpux_make_needs_suffix_list

# Command-line flag to silence nested $(MAKE).
$(VERBOSE)MAKESILENT = -s

#Suppress display of executed commands.
$(VERBOSE).SILENT:

# A target that is always out of date.
cmake_force:
.PHONY : cmake_force

#=============================================================================
# Set environment variables for the build.

# The shell in which to execute make rules.
SHELL = /bin/sh

# The CMake executable.
CMAKE_COMMAND = /usr/bin/cmake

# The command to remove a file.
RM = /usr/bin/cmake -E rm -f

# Escaping for special characters.
EQUALS = =

# The top-level source directory on which CMake was run.
CMAKE_SOURCE_DIR = /root/code/01/10/libxml2-rust

# The top-level build directory on which CMake was run.
CMAKE_BINARY_DIR = /root/code/01/10/libxml2-rust

# Include any dependencies generated for this target.
include CMakeFiles/testAutomata.dir/depend.make
# Include any dependencies generated by the compiler for this target.
include CMakeFiles/testAutomata.dir/compiler_depend.make

# Include the progress variables for this target.
include CMakeFiles/testAutomata.dir/progress.make

# Include the compile flags for this target's objects.
include CMakeFiles/testAutomata.dir/flags.make

CMakeFiles/testAutomata.dir/testAutomata.c.o: CMakeFiles/testAutomata.dir/flags.make
CMakeFiles/testAutomata.dir/testAutomata.c.o: testAutomata.c
CMakeFiles/testAutomata.dir/testAutomata.c.o: CMakeFiles/testAutomata.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --progress-dir=/root/code/01/10/libxml2-rust/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Building C object CMakeFiles/testAutomata.dir/testAutomata.c.o"
	/usr/bin/cc $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -MD -MT CMakeFiles/testAutomata.dir/testAutomata.c.o -MF CMakeFiles/testAutomata.dir/testAutomata.c.o.d -o CMakeFiles/testAutomata.dir/testAutomata.c.o -c /root/code/01/10/libxml2-rust/testAutomata.c

CMakeFiles/testAutomata.dir/testAutomata.c.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Preprocessing C source to CMakeFiles/testAutomata.dir/testAutomata.c.i"
	/usr/bin/cc $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -E /root/code/01/10/libxml2-rust/testAutomata.c > CMakeFiles/testAutomata.dir/testAutomata.c.i

CMakeFiles/testAutomata.dir/testAutomata.c.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Compiling C source to assembly CMakeFiles/testAutomata.dir/testAutomata.c.s"
	/usr/bin/cc $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -S /root/code/01/10/libxml2-rust/testAutomata.c -o CMakeFiles/testAutomata.dir/testAutomata.c.s

# Object files for target testAutomata
testAutomata_OBJECTS = \
"CMakeFiles/testAutomata.dir/testAutomata.c.o"

# External object files for target testAutomata
testAutomata_EXTERNAL_OBJECTS =

testAutomata: CMakeFiles/testAutomata.dir/testAutomata.c.o
testAutomata: CMakeFiles/testAutomata.dir/build.make
testAutomata: libxml2.a
testAutomata: /usr/lib64/libc.so
testAutomata: /usr/lib64/liblzma.so
testAutomata: /usr/lib64/libz.so
testAutomata: CMakeFiles/testAutomata.dir/link.txt
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --bold --progress-dir=/root/code/01/10/libxml2-rust/CMakeFiles --progress-num=$(CMAKE_PROGRESS_2) "Linking C executable testAutomata"
	$(CMAKE_COMMAND) -E cmake_link_script CMakeFiles/testAutomata.dir/link.txt --verbose=$(VERBOSE)

# Rule to build all files generated by this target.
CMakeFiles/testAutomata.dir/build: testAutomata
.PHONY : CMakeFiles/testAutomata.dir/build

CMakeFiles/testAutomata.dir/clean:
	$(CMAKE_COMMAND) -P CMakeFiles/testAutomata.dir/cmake_clean.cmake
.PHONY : CMakeFiles/testAutomata.dir/clean

CMakeFiles/testAutomata.dir/depend:
	cd /root/code/01/10/libxml2-rust && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /root/code/01/10/libxml2-rust /root/code/01/10/libxml2-rust /root/code/01/10/libxml2-rust /root/code/01/10/libxml2-rust /root/code/01/10/libxml2-rust/CMakeFiles/testAutomata.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : CMakeFiles/testAutomata.dir/depend
