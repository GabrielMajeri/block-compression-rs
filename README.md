# block-compression

[![Build Status](https://travis-ci.org/GuildMasterInfinite/block-compression-rs.svg?branch=master)](https://travis-ci.org/GuildMasterInfinite/block-compression-rs)
[![codecov](https://codecov.io/gh/GuildMasterInfinite/block-compression-rs/branch/develop/graph/badge.svg)](https://codecov.io/gh/GuildMasterInfinite/block-compression-rs)
[![Build status](https://ci.appveyor.com/api/projects/status/ycp5ilof3fwe6mew?svg=true)](https://ci.appveyor.com/project/GuildMasterInfinite/block-compression-rs)

Tools for encoding / decoding block-compression files. 

Block compression is another name for the encoding DDS files use (where it is called DXT - DirectX Texture Compression) and S3TC (S3 Texture Compression).

The aim of this library is to support all of the currently released BC specifications (BC 1 through 7).

The crate is named "block-compression" because this name is more general and doesn't lean towards Direct3D nor OpenGL / Vulkan.

## Feature matrix
| Identifier | Encoding | Decoding |
|-|-|-|
| BC 1 | X | X |
| BC 2 | X | X |
| BC 3 | X | X |
| BC 4 | X | X |
| BC 5 | X | X |
| BC 6 | X | X |
| BC 7 | X | X |

## License
Dual-licensed MIT or Apache 2.0, at the library user's leisure.
