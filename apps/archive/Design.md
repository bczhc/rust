## Archive

**An archive format for data backups with indexing and compression capabilities**

Version: 1

---

| header | entry 1 | entry checksum | ...  | entry N | entry checksum | file 1 | ...  | file N |
| ------ | ------- | -------------- | ---- | ------- | -------------- | ------ | ---- | ------ |

Please refer to `archive::Header` and `archive::Entry`.


About path strings:

On Unix things are very easy: paths can be in arbitrary binary (meaning no need
to be valid UTF-8 encoded sequences).
So I just use `OsString` with unix-specific "from [u8]"-like functions;

But I'm not familiar with this on Windows. I don't know how Windows handles
these. Now it will panic when meeting any invalid UTF-8.

And I know these restrictions are related to filesystems. For example ext4
and Btrfs allow arbitrary binary except '\0' as file names, and I don't know how
Windows will handle this when using some methods to mount these filesystems
on a Windows platform.
