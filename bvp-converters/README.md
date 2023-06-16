# BVP conversion tools
Here are located various tools for conversion between volumes in BVP format and other formats.

## Tools
Currently, the following programs are included:

* `raw2bvp` - Converts volume in raw data file to BVP
* `bvp2raw` - Converts volume in BVP format to raw data file

## raw2bvp
The program accepts one parameter: a path to JSON configuration file. The contents of the configuration file are a JSON object with the following attributes:

| **Option**      | **Type**  | **Description**                                                                                               | **Required** |
|-----------------|-----------|---------------------------------------------------------------------------------------------------------------|--------------|
| inputFile       | str       | A path to a raw data file of the volume                                                                       | yes          |
| outputFile      | str       | A path to final result file.                                                                                  | yes          |
| dimensions      | arr[uint] | An array of 3 positive integers, representing dimensions of the input volume                                  | yes          |
| blockDimensions | arr[uint] | An array of 3 positive integers, representing dimensions of blocks to chop the input volume into              | yes          |
| format          | object    | Represents the interpretation of data for conversion to BVP                                                   | yes          |
| archive         | str       | If provided, combines all output files into archive of the provided type. So far, SAF and None are supported  | no           |
| compression     | str       | If provided, compresses output data with provided compression algorithm. So far, LZ4S and None are supported. | no           |
| name            | str       | A custom name to be set as BVP `modality` attribute. Defaults to the original file name (without extension)   | no           |
| description     | str       | A custom description to be set as BVP `modality` attribute. Defaults to none                                  | no           |
| semanticType    | str       | A custom semantic type to be set as BVP `modality` attribute. Defaults to none                                | no           |
| volumeScale     | arr[f32]  | Sets volume size in real life (in millimeters). Defaults to [1, 1, 1]                                         | no           |
| voxelScale      | arr[f32]  | Sets voxel size in real life (in millimeters). Defaults to none                                               | no           |
| author          | str       | Sets the author of the volume(s) in BVP asset. Defaults to none                                               | no           |
| copyright       | str       | Sets the copyright of the volume(s) in BVP asset. Defaults to none                                            | no           |
| acquisitionTime | str       | Sets the acquisition time of the BVP asset. Should be in timestamp format. Defaults to none                   | no           |

Of the formats, only `mono` is currently supported. The corresponding object might look like this:

```json
{
    "family": "mono",
    "count": uint,
    "size": uint,
    "type": str // "i", "u", and "f" are supported
}
```

If the `archive` option is not provided or has the value `"None"`, the files are not archived and are instead output at the location of the configuration file.

## bvp2raw
The program can be executed as follows:

```
bvp2raw <input_file> <archive_type>
```

* input_file - a file or folder containing BVP data (manifest and block data)
* archive_type - a type of archive that is used. If omitted, it is read as directory. Currently, `SAF` and `ZIP` are supported.

The help message can also be viewed with `--help` flag.

The program outputs volume in raw data format.

## Building from source

Moving into `bvp-converters` folder, the binaries can be generated with `cargo build --release`, and are afterwards present in `/target/release` folder.