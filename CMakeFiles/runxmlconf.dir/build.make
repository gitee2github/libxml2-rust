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
CMAKE_SOURCE_DIR = /root/code/01/01/libxml2-rust

# The top-level build directory on which CMake was run.
CMAKE_BINARY_DIR = /root/code/01/01/libxml2-rust

# Include any dependencies generated for this target.
include CMakeFiles/runxmlconf.dir/depend.make
# Include any dependencies generated by the compiler for this target.
include CMakeFiles/runxmlconf.dir/compiler_depend.make

# Include the progress variables for this target.
include CMakeFiles/runxmlconf.dir/progress.make

# Include the compile flags for this target's objects.
include CMakeFiles/runxmlconf.dir/flags.make

CMakeFiles/runxmlconf.dir/runxmlconf.c.o: CMakeFiles/runxmlconf.dir/flags.make
CMakeFiles/runxmlconf.dir/runxmlconf.c.o: runxmlconf.c
CMakeFiles/runxmlconf.dir/runxmlconf.c.o: CMakeFiles/runxmlconf.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --progress-dir=/root/code/01/01/libxml2-rust/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Building C object CMakeFiles/runxmlconf.dir/runxmlconf.c.o"
	/usr/bin/cc $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -MD -MT CMakeFiles/runxmlconf.dir/runxmlconf.c.o -MF CMakeFiles/runxmlconf.dir/runxmlconf.c.o.d -o CMakeFiles/runxmlconf.dir/runxmlconf.c.o -c /root/code/01/01/libxml2-rust/runxmlconf.c

CMakeFiles/runxmlconf.dir/runxmlconf.c.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Preprocessing C source to CMakeFiles/runxmlconf.dir/runxmlconf.c.i"
	/usr/bin/cc $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -E /root/code/01/01/libxml2-rust/runxmlconf.c > CMakeFiles/runxmlconf.dir/runxmlconf.c.i

CMakeFiles/runxmlconf.dir/runxmlconf.c.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Compiling C source to assembly CMakeFiles/runxmlconf.dir/runxmlconf.c.s"
	/usr/bin/cc $(C_DEFINES) $(C_INCLUDES) $(C_FLAGS) -S /root/code/01/01/libxml2-rust/runxmlconf.c -o CMakeFiles/runxmlconf.dir/runxmlconf.c.s

# Object files for target runxmlconf
runxmlconf_OBJECTS = \
"CMakeFiles/runxmlconf.dir/runxmlconf.c.o"

# External object files for target runxmlconf
runxmlconf_EXTERNAL_OBJECTS =

runxmlconf: CMakeFiles/runxmlconf.dir/runxmlconf.c.o
runxmlconf: CMakeFiles/runxmlconf.dir/build.make
runxmlconf: libxml2.a
runxmlconf: /usr/lib64/libc.so
runxmlconf: /usr/lib64/liblzma.so
runxmlconf: /usr/lib64/libz.so
runxmlconf: CMakeFiles/runxmlconf.dir/link.txt
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --bold --progress-dir=/root/code/01/01/libxml2-rust/CMakeFiles --progress-num=$(CMAKE_PROGRESS_2) "Linking C executable runxmlconf"
	$(CMAKE_COMMAND) -E cmake_link_script CMakeFiles/runxmlconf.dir/link.txt --verbose=$(VERBOSE)

# Rule to build all files generated by this target.
CMakeFiles/runxmlconf.dir/build: runxmlconf
.PHONY : CMakeFiles/runxmlconf.dir/build

CMakeFiles/runxmlconf.dir/clean:
	$(CMAKE_COMMAND) -P CMakeFiles/runxmlconf.dir/cmake_clean.cmake
.PHONY : CMakeFiles/runxmlconf.dir/clean

CMakeFiles/runxmlconf.dir/depend:
	cd /root/code/01/01/libxml2-rust && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /root/code/01/01/libxml2-rust /root/code/01/01/libxml2-rust /root/code/01/01/libxml2-rust /root/code/01/01/libxml2-rust /root/code/01/01/libxml2-rust/CMakeFiles/runxmlconf.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : CMakeFiles/runxmlconf.dir/depend

