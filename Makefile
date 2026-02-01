BUILD_DIR := build

.PHONY: build clean

build:
	mkdir -p $(BUILD_DIR)
	python3 -m markdown README.md > $(BUILD_DIR)/index.html

clean:
	rm -rf $(BUILD_DIR)
