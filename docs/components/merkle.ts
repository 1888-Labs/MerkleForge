// Browser-side Merkle hashing used by the demo and examples pages.
/**
 * Browser mirror of `BinaryMerkleTree<Sha256>`: leaves are `SHA-256(0x00 || data)`,
 * internal nodes are `SHA-256(0x01 || left || right)`, and the leaf layer is padded
 * to a power of two with `Sha256::empty()` (the hash of an empty leaf).
 */

export type ProofSide = "Left" | "Right";

export type DemoResult = {
  root: string;
  leafHash: string;
  proof: Array<{ side: ProofSide; hash: string }>;
  verified: boolean;
  recomputedRoot: string;
  targetLeaf: string;
  leafIndex: number;
  leafCount: number;
};

type ProofNode = { side: ProofSide; hash: Uint8Array };

const encoder = new TextEncoder();

function concatBytes(...chunks: Uint8Array[]): Uint8Array {
  const merged = new Uint8Array(chunks.reduce((total, chunk) => total + chunk.length, 0));
  let offset = 0;
  for (const chunk of chunks) {
    merged.set(chunk, offset);
    offset += chunk.length;
  }
  return merged;
}

export function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes, byte => byte.toString(16).padStart(2, "0")).join("");
}

async function sha256(bytes: Uint8Array): Promise<Uint8Array> {
  const copy = new Uint8Array(bytes);
  return new Uint8Array(await crypto.subtle.digest("SHA-256", copy.buffer as ArrayBuffer));
}

function nextPowerOfTwo(value: number): number {
  return 2 ** Math.ceil(Math.log2(Math.max(value, 1)));
}

export function hashLeaf(data: string): Promise<Uint8Array> {
  return sha256(concatBytes(new Uint8Array([0x00]), encoder.encode(data)));
}

export function hashNode(left: Uint8Array, right: Uint8Array): Promise<Uint8Array> {
  return sha256(concatBytes(new Uint8Array([0x01]), left, right));
}

async function buildLayers(leaves: string[]): Promise<Uint8Array[][]> {
  const emptyLeaf = await hashLeaf("");
  const padded = await Promise.all(leaves.map(hashLeaf));
  while (padded.length < nextPowerOfTwo(leaves.length)) padded.push(emptyLeaf);

  async function buildRemainingLayers(current: Uint8Array[]): Promise<Uint8Array[][]> {
    if (current.length === 1) return [current];

    const parents = await Promise.all(
      Array.from({ length: current.length / 2 }, (_, index) =>
        hashNode(current[index * 2], current[index * 2 + 1]),
      ),
    );
    return [current, ...(await buildRemainingLayers(parents))];
  }

  return buildRemainingLayers(padded);
}

function proofForIndex(layers: Uint8Array[][], leafIndex: number): ProofNode[] {
  const proof: ProofNode[] = [];
  let index = leafIndex;
  for (const layer of layers.slice(0, -1)) {
    const isLeftChild = index % 2 === 0;
    proof.push({ side: isLeftChild ? "Right" : "Left", hash: layer[isLeftChild ? index + 1 : index - 1] });
    index = Math.floor(index / 2);
  }
  return proof;
}

async function recomputeRoot(leaf: string, proof: ProofNode[]): Promise<Uint8Array> {
  let current = await hashLeaf(leaf);
  current = await proof.reduce(
    async (hashPromise, node) => {
      const hash = await hashPromise;
      return node.side === "Left" ? hashNode(node.hash, hash) : hashNode(hash, node.hash);
    },
    Promise.resolve(current),
  );
  return current;
}

/**
 * Builds a tree from `leaves`, proves `requestedIndex`, and verifies `claimedLeaf`
 * (defaults to the real leaf) against the root using only the proof.
 */
export async function runProof(leaves: string[], requestedIndex: number, claimedLeaf?: string): Promise<DemoResult> {
  if (leaves.length === 0) throw new Error("Add at least one row before building a proof.");

  const leafIndex = Math.min(Math.max(requestedIndex, 0), leaves.length - 1);
  const layers = await buildLayers(leaves);
  const root = bytesToHex(layers.at(-1)![0]);
  const proof = proofForIndex(layers, leafIndex);
  const targetLeaf = claimedLeaf ?? leaves[leafIndex];
  const recomputedRoot = bytesToHex(await recomputeRoot(targetLeaf, proof));

  return {
    root,
    leafHash: bytesToHex(await hashLeaf(targetLeaf)),
    proof: proof.map(node => ({ side: node.side, hash: bytesToHex(node.hash) })),
    verified: root === recomputedRoot,
    recomputedRoot,
    targetLeaf,
    leafIndex,
    leafCount: leaves.length,
  };
}
