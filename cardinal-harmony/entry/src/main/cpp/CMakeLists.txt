# the minimum version of CMake.
cmake_minimum_required(VERSION 3.5.0)
project(cardianl-harmony)

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTORCC ON)
set(NATIVERENDER_ROOT_PATH ${CMAKE_CURRENT_SOURCE_DIR})

list(APPEND CMAKE_FIND_ROOT_PATH ${QT_PREFIX})

if(DEFINED PACKAGE_FIND_FILE)
    include(${PACKAGE_FIND_FILE})
endif()

include_directories(${NATIVERENDER_ROOT_PATH}
                    ${NATIVERENDER_ROOT_PATH}/include)

find_package(QT NAMES Qt5 Qt6 REQUIRED COMPONENTS Core)
find_package(Qt${QT_VERSION_MAJOR} REQUIRED COMPONENTS Core Gui Widgets)

add_library(entry SHARED main.cpp)
target_link_libraries(entry PRIVATE Qt${QT_VERSION_MAJOR}::Core
Qt${QT_VERSION_MAJOR}::Gui
Qt${QT_VERSION_MAJOR}::Widgets
Qt${QT_VERSION_MAJOR}::QOpenHarmonyPlatformIntegrationPlugin
)