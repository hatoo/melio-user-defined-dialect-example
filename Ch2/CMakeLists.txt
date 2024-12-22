cmake_minimum_required(VERSION 3.20.0)

project(ToyDialect)
find_package(LLVM REQUIRED CONFIG)
find_package(MLIR REQUIRED CONFIG)

message(STATUS "Found LLVM ${LLVM_PACKAGE_VERSION}")
message(STATUS "Using LLVMConfig.cmake in: ${LLVM_DIR}")

get_filename_component(MLIR_DIR_ABSOLUTE ${MLIR_DIR}
  REALPATH BASE_DIR ${CMAKE_CURRENT_BINARY_DIR})
list(APPEND CMAKE_MODULE_PATH ${MLIR_DIR_ABSOLUTE})

include_directories(${LLVM_INCLUDE_DIRS})

include(AddLLVM)
include(AddMLIR)
include(TableGen)

add_subdirectory(include)
add_library(toyc-ch2
  parser/AST.cpp
  mlir/MLIRGen.cpp
  mlir/Dialect.cpp
  mlir/Dialects.cpp
  )
add_dependencies(toyc-ch2 ToyCh2OpsIncGen)
include_directories(include/)

install (TARGETS toyc-ch2 DESTINATION .)