# Define the source directories
SRC_DIRS := fixtures/z80/simple fixtures/z80/stack fixtures/z80/test_call

# Find all .asm files in the source directories
ASM_SRCS := $(foreach dir, $(SRC_DIRS), $(wildcard $(dir)/*.asm))

# Define the output .bin files
BIN_OBJS := $(ASM_SRCS:.asm=.bin)

# Default target
all: $(BIN_OBJS)

# Compile .asm files to .bin files
%.bin: %.asm
	z80asm -o $@ $<

# Clean up the output files
clean:
	rm -f $(BIN_OBJS)
