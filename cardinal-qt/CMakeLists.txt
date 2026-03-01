cmake_minimum_required(VERSION 3.22.0)
project(Helloworld)

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTORCC ON)

include(FetchContent)

FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    # GIT_TAG v0.6 # Optionally specify a commit hash, version tag or branch here
)
# Set any global configuration variables such as `Rust_TOOLCHAIN` before this line!
FetchContent_MakeAvailable(Corrosion)

set(NATIVERENDER_ROOT_PATH ${CMAKE_CURRENT_SOURCE_DIR})
set(ARGS --target-dir ${CMAKE_BINARY_DIR}/target)

list(APPEND CMAKE_FIND_ROOT_PATH ${QT_PREFIX})
include_directories(${NATIVERENDER_ROOT_PATH}
                    ${NATIVERENDER_ROOT_PATH}/include)

find_package(QT NAMES Qt5 Qt6 REQUIRED COMPONENTS Core Gui Widgets )
find_package(Qt${QT_VERSION_MAJOR} REQUIRED COMPONENTS Gui Widgets )

corrosion_import_crate(MANIFEST_PATH ${CMAKE_SOURCE_DIR}/Cargo.toml)


# if (CMAKE_BUILD_TYPE STREQUAL "Release")
#     message(STATUS "[RUST] Detected release build")
#     set(build_profile "release")
#     set(ARGS ${ARGS} --release)
# else()
#     message(STATUS "[RUST] Using default debug build")
#     set(build_profile "debug")
# endif()
# set(ENV{QMAKE} ${QMAKE})
# add_custom_target(
#     cargo_build ALL
#     COMMAND
#         ${CMAKE_COMMAND} -E env QMAKE=${QMAKE}
#         ohrs cargo -a x86_64 -- build ${ARGS}
#     SOURCE src/lib.rs
# )
# add_library(cardinal_qt_rs SHARED IMPORTED)
#set_property(TARGET rust_lib PROPERTY IMPORTED_LOCATION "${CMAKE_BINARY_DIR}/target/${build_profile}/libcardinal_qt_rs.so")

add_executable(entry src/main.cpp)

target_link_libraries(entry PUIBLIC
Qt${QT_VERSION_MAJOR}::Core
Qt${QT_VERSION_MAJOR}::Gui
Qt${QT_VERSION_MAJOR}::Widgets
# Qt${QT_VERSION_MAJOR}::QOpenHarmonyPlatformIntegrationPlugin
cardinal_qt_rs
)

#target_link_libraries(entry INTERFACE cardinal_qt_rs )
