# Environment Variables
## Compile-Time variables
| Name          | Optional | Default Value | Accepted Values    | Description                                       |
|---------------|----------|---------------|--------------------|---------------------------------------------------|
| TARGET_VOLUME | yes      | 0.1           | 0.0..=1.0          | The target volume of normalized songs (nonlinear) |
| ENCODING      | yes      | gzip          | brotli, gzip, none | The encoding used to encode embedded files with   |

## Runtime Variables
| Name    | Optional | Default Value | Accepted Values | Description                                            |
|---------|----------|---------------|-----------------|--------------------------------------------------------|
| LOGGING | yes      | false         | true, false     | Whether to log console into a file called `latest.log` |
| DEBUG   | yes      | false         | true, false     | Whether to display some debug information              |
