# CMAKE generated file: DO NOT EDIT!
# Generated by "Unix Makefiles" Generator, CMake Version 3.23

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
CMAKE_COMMAND = /snap/cmake/1088/bin/cmake

# The command to remove a file.
RM = /snap/cmake/1088/bin/cmake -E rm -f

# Escaping for special characters.
EQUALS = =

# The top-level source directory on which CMake was run.
CMAKE_SOURCE_DIR = /root/pr/20220707/libxml2-rust/libxml2-2.9.12_github_version

# The top-level build directory on which CMake was run.
CMAKE_BINARY_DIR = /root/pr/20220707/libxml2-rust/libxml2-2.9.12_github_version

# Include any dependencies generated for this target.
include CMakeFiles/testRelax.dir/depend.make
# Include any dependencies generated by the compiler for this target.
include CMakeFiles/testRelax.dir/compiler_depend.make

# Include the progress variables for this target.
include CMakeFiles/testRelax.dir/progress.make

# Include the compile flags for this target's objects.
include CMakeFiles/testRelax.dir/flags.make

CMakeFiles/testRelax.dir/testRelax.c.o: CMakeFiles/testRelax.dir/flags.make
CMakeFiles/testRelax.dir/testRelax.c.o: testRelax.c
CMakeFiles/testRelax.dir/testRelax.c.o: CMakeFiles/testRelax.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --progress-dir=/root/pr/20220707/libxml2-rust/libxml2-2.9.12_github_version/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Building C object CMakeFiles/testRelax.dir/testRelax.c.o"
	/usr/bin/cc $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -MD -MT CMakeFiles/testRelax.dir/testRelax.c.o -MF CMakeFiles/testRelax.dir/testRelax.c.o.d -o CMakeFiles/testRelax.dir/testRelax.c.o -c /root/pr/20220707/libxml2-rust/libxml2-2.9.12_github_version/testRelax.c

CMakeFiles/testRelax.dir/testRelax.c.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Preprocessing C source to CMakeFiles/testRelax.dir/testRelax.c.i"
	/usr/bin/cc $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -E /root/pr/20220707/libxml2-rust/libxml2-2.9.12_github_version/testRelax.c > CMakeFiles/testRelax.dir/testRelax.c.i

CMakeFiles/testRelax.dir/testRelax.c.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Compiling C source to assembly CMakeFiles/testRelax.dir/testRelax.c.s"
	/usr/bin/cc $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -S /root/pr/20220707/libxml2-rust/libxml2-2.9.12_github_version/testRelax.c -o CMakeFiles/testRelax.dir/testRelax.c.s

# Object files for target testRelax
testRelax_OBJECTS = \
"CMakeFiles/testRelax.dir/testRelax.c.o"

# External object files for target testRelax
testRelax_EXTERNAL_OBJECTS =

testRelax: CMakeFiles/testRelax.dir/testRelax.c.o
testRelax: CMakeFiles/testRelax.dir/build.make
testRelax: libxml2.a
testRelax: /usr/lib/x86_64-linux-gnu/liblzma.so
testRelax: /usr/lib/x86_64-linux-gnu/libz.so
testRelax: CMakeFiles/testRelax.dir/link.txt
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --bold --progress-dir=/root/pr/20220707/libxml2-rust/libxml2-2.9.12_github_version/CMakeFiles --progress-num=$(CMAKE_PROGRESS_2) "Linking C executable testRelax"
	$(CMAKE_COMMAND) -E cmake_link_script CMakeFiles/testRelax.dir/link.txt --verbose=$(VERBOSE)

# Rule to build all files generated by this target.
CMakeFiles/testRelax.dir/build: testRelax
.PHONY : CMakeFiles/testRelax.dir/build

CMakeFiles/testRelax.dir/clean:
	$(CMAKE_COMMAND) -P CMakeFiles/testRelax.dir/cmake_clean.cmake
.PHONY : CMakeFiles/testRelax.dir/clean

CMakeFiles/testRelax.dir/depend:
	cd /root/pr/20220707/libxml2-rust/libxml2-2.9.12_github_version && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /root/pr/20220707/libxml2-rust/libxml2-2.9.12_github_version /root/pr/20220707/libxml2-rust/libxml2-2.9.12_github_version /root/pr/20220707/libxml2-rust/libxml2-2.9.12_github_version /root/pr/20220707/libxml2-rust/libxml2-2.9.12_github_version /root/pr/20220707/libxml2-rust/libxml2-2.9.12_github_version/CMakeFiles/testRelax.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : CMakeFiles/testRelax.dir/depend

