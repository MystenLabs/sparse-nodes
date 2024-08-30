## Sparse Nodes

```update_streams([stream_id, [point_1, point_2, ..., point_n]]) -> MerkleTreeDigest. A function called at every checkpoint to update the streams with new data points. Returns the digest of the sparse node ADS to be included in the checkpoint.```

- Counters: For each stream, calculate `lc = |[point_1, point_2, ..., point_n]|, gc = lc + past_count`. Include `(lc, gc, point_n)` in a Merkle tree.

- Hash Chains: For each stream, calculate `new_head = H(point_n, (... H(point_1, past_head))...)`. Include `(new_head, point_n)` in a Merkle tree.

- M-HC: For each stream, calculate local_digest = `MT-digest([point_1, point_2, ..., point_n]), head = H(local_digest, past_head)`. Include `(local_digest, head, point_n)` in a Merkle tree.