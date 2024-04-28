package main

//	#include <stdint.h>
//	#include <stdbool.h>
//	typedef struct RuneEntry {
//		uint64_t block;
//		uint64_t burned_low;
//		uint64_t burned_high;
//		uint8_t divisibility;
//		char* etching;
//		uint64_t mints_low;
//		uint64_t mints_high;
//		uint64_t number;
//		uint64_t premine_low;
//		uint64_t premine_high;
//		char symbol;
//		uint64_t timestamp;
//		bool turbo;
//	} RuneEntry;
//	typedef struct RuneID {
//		uint64_t block;
//		uint32_t tx;
//	} RuneID;
//	typedef struct TapLockConfig {
//		void* transaction;
//		int transaction_length;
//		RuneEntry* rune_entry;
//		uint64_t amount;
//		char* script_key;
//		char* batch_key;
//		RuneID* rune_id;
//	} TapLockConfig;
//	typedef struct TapLockResult {
//		void* transaction;
//		int transaction_length;
//		char* asset_id;
//	} TapLockResult;
import "C"
import (
	"bytes"
	"fmt"
	"unsafe"

	"github.com/btcsuite/btcd/btcec/v2"
	"github.com/btcsuite/btcd/btcutil/psbt"
	"github.com/btcsuite/btcd/txscript"
	"github.com/btcsuite/btcd/wire"
	"github.com/lightninglabs/taproot-assets/asset"
	"github.com/lightninglabs/taproot-assets/commitment"
	"github.com/lightninglabs/taproot-assets/tapgarden"
	"github.com/lightninglabs/taproot-assets/tapscript"
	"github.com/lightningnetwork/lnd/keychain"
)

const AnchorIndex = 2

// wrapper focused on slices over reader/writes.
type Tx struct {
	wire.MsgTx
}

func (tx *Tx) Unmarshal(data []byte) error {
	b := bytes.NewBuffer(data)

	return tx.MsgTx.Deserialize(b)
}

func (tx *Tx) Marshal() ([]byte, error) {
	b := new(bytes.Buffer)

	err := tx.MsgTx.Serialize(b)
	if err != nil {
		return nil, err
	}

	return b.Bytes(), nil
}

type Block struct {
	wire.MsgBlock
}

func (blk *Block) Unmarshal(data []byte) error {
	b := bytes.NewBuffer(data)

	return blk.MsgBlock.Deserialize(b)
}

type Packet struct {
	*psbt.Packet
}

func (p *Packet) Unmarshal(data []byte) error {
	b := bytes.NewBuffer(data)

	var err error

	p.Packet, err = psbt.NewFromRawBytes(b, true)

	return err
}

func mintingKey(commit *commitment.TapCommitment, batch *btcec.PublicKey) (*btcec.PublicKey, error) {
	root := commit.TapscriptRoot(nil)
	pub := txscript.ComputeTaprootOutputKey(batch, root[:])

	return pub, nil
}

func commit(id *C.RuneID, amount uint64, script *btcec.PublicKey, tx *wire.MsgTx) (*commitment.TapCommitment, *asset.Asset) {
	// create the asset seedling.
	seed := tapgarden.Seedling{
		AssetVersion: asset.V1,
		AssetType:    asset.Normal,
		AssetName:    fmt.Sprintf("%d:%d", id.block, id.tx),

		Amount: amount,
	}

	// extract the genesis point.
	gp := tx.TxIn[0].PreviousOutPoint

	// create the asset and tap commitment.
	// create the genesis.
	gen := asset.Genesis{
		FirstPrevOut: gp,
		Tag:          seed.AssetName,
		OutputIndex:  AnchorIndex,
		Type:         seed.AssetType,
	}

	// populate the metadata.
	if seed.Meta != nil {
		gen.MetaHash = seed.Meta.MetaHash()
	}

	// create a fake key descriptor.
	key := keychain.KeyDescriptor{
		PubKey: script,
	}

	// create the tap-tweak key.
	twk := asset.NewScriptKeyBip86(key)

	// create the asset.
	ast, err := asset.New(
		gen,
		seed.Amount,
		0,
		0,
		twk,
		nil,
		asset.WithAssetVersion(seed.AssetVersion),
	)

	// create the MSST commitment.
	cmt, err := commitment.FromAssets(ast)
	if err != nil {
		panic(err)
	}

	return cmt, ast
}

func parseKey(p *C.char) *btcec.PublicKey {
	k, err := btcec.ParsePubKey(unsafe.Slice((*byte)(unsafe.Pointer(p)), 33))
	if err != nil {
		panic(err)
	}

	return k
}

// lock creates a taproot asset mint from the rune burn.
//
//export TapLock
func TapLock(cfg *C.TapLockConfig) uintptr {
	rtx := unsafe.Slice((*byte)(cfg.transaction), cfg.transaction_length)

	var tx Tx

	// decode the transaction.
	err := tx.Unmarshal(rtx)
	if err != nil {
		panic(err)
	}

	// parse the script key.
	sk := parseKey(cfg.script_key)

	// create the asset and commitment.
	cmt, ast := commit(cfg.rune_id, uint64(cfg.amount), sk, &tx.MsgTx)

	// parse the batch key.
	bk := parseKey(cfg.batch_key)

	// derive the output from the commitment and batch key.
	mk, err := mintingKey(cmt, bk)
	if err != nil {
		panic(err)
	}

	// convert the minting key to a mint to taproot script.
	p2tr, err := tapscript.PayToTaprootScript(mk)
	if err != nil {
		panic(err)
	}

	// append the taproot output.
	tx.TxOut = append(tx.TxOut, &wire.TxOut{
		PkScript: p2tr,
	})

	// encode the transaction.
	r, err := tx.Marshal()
	if err != nil {
		panic(err)
	}

	return uintptr(unsafe.Pointer(&C.TapLockResult{
		transaction:        unsafe.Pointer(&r[0]),
		transaction_length: C.int(len(r)),
		asset_id:           C.CString(ast.ID().String()),
	}))
}

func main() {}
