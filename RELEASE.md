# Release Process

This document outlines the process for creating a new release of Trigram.

## Pre-Release Checklist

1. Ensure all tests pass locally:

   ```bash
   mix test
   mix credo
   mix dialyzer
   ```

2. Update version number in `mix.exs`:

   ```elixir
   @version "X.Y.Z"
   ```

3. Update version number in `README.md` installation example if needed.

4. Commit version changes:
   ```bash
   git add mix.exs README.md
   git commit -m "Bump version to X.Y.Z"
   git push origin main
   ```

## Creating the Release

1. Create and push a git tag:

   ```bash
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

2. The GitHub Actions workflow will automatically:
   - Build precompiled NIFs for all target platforms
   - Create a GitHub release with the artifacts
   - Upload the precompiled binaries

3. Monitor the GitHub Actions workflow to ensure all builds complete successfully:
   - Go to: https://github.com/EnaiaInc/trigram/actions
   - Wait for all matrix jobs to complete

## Generating Checksums

After the GitHub release is created and all precompiled binaries are uploaded:

1. Download all precompiled artifacts and generate checksums:

   ```bash
   mix rustler_precompiled.download Trigram.Native --all --print
   ```

2. This command will:
   - Download all precompiled NIFs from the GitHub release
   - Calculate SHA256 checksums for each binary
   - Print the checksum configuration in Elixir format

3. Copy the output and create/update the checksum file:

   ```bash
   # The output will look like:
   # @checksums %{
   #   "nif-2.15-aarch64-apple-darwin" => "sha256:...",
   #   "nif-2.15-aarch64-unknown-linux-gnu" => "sha256:...",
   #   ...
   # }
   ```

4. Save the checksums to the appropriate checksum file
   (e.g., `checksum-Elixir.Trigram.Native.exs`).

5. Keep the checksum file untracked; it only needs to exist locally
   when running `mix hex.publish` so it gets packaged.

## Publishing to Hex

1. Ensure the checksum file exists locally.

2. Publish the package to Hex:

   ```bash
   mix hex.publish
   ```

3. Review the files that will be published (should include checksum files).

4. Confirm the publication.

## Post-Release

1. Verify the package on Hex.pm: https://hex.pm/packages/trigram

2. Test installation in a separate project:

   ```bash
   # In a test project
   mix deps.get
   # Verify it downloads precompiled NIFs instead of compiling
   ```

3. Update any dependent projects or documentation as needed.

## Troubleshooting

### Checksum Generation Fails

If `mix rustler_precompiled.download` fails:

- Ensure the GitHub release exists and all artifacts are uploaded
- Check that the version in `mix.exs` matches the git tag
- Verify the `base_url` in `lib/trigram/native.ex` is correct

### Missing Precompiled Binaries

If some platform builds fail:

- Check the GitHub Actions logs for specific errors
- Re-run failed jobs if it was a transient error
- Update the release notes to document any unsupported platforms

### Hex Publish Fails

If `mix hex.publish` fails:

- Ensure you're authenticated: `mix hex.user auth`
- Check that all files in the `files` list in `mix.exs` exist
- Verify the checksum file is included in the package files list
