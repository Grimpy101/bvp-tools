# BVP conversion tools
Here are located various tools for conversion between volumes in BVP format and other formats.

## Tools
Currently, there are the following programs:

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
| format          | Object    | Represents the interpretation of data for conversion to BVP                                                   | yes          |
| archive         | str       | If provided, combines all output files into archive of the provided type. So far, SAF and None are supported  | no           |
| compression     | str       | If provided, compresses output data with provided compression algorithm. So far, LZ4S and None are supported. | no           |

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
The program accepts one parameter: the path to BVP archive file, or the path to `manifest.json` file.

It outputs volume in raw data format.

## Building from source

Moving into the `bvp-converters`, the binaries can be generated with `cargo build --release`, and are consequently present in `/target/release` folder.