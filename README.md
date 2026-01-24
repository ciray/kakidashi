# kakidashi

## CSV圧縮
```bash
$ cd src/resources
$ cat data.csv | grep -v ",," | gzip > data.csv.gz
```
