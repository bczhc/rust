## Design
| header | track info size | track info | track 1 | track 2 | ...  | track N |
| ------ | --------------- | ---------- | ------- | ------- | ---- | ------- |

- header: b"bczhc image-track"

- track info size: u32, little endian

- track info: JSON text, indicates the following fields of tracks:

  ```rust
  pub struct Track {
      number: u32,
      start_sector: u64,
      end_sector: u64,
      used_sectors: u64,
      content_size: u64,
  }
  ```

- track N: file content, padded if not a multiple of sector size

