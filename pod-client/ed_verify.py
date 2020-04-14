import ed25519
import sys

if len(sys.argv) < 4:
    print("Usage: " + sys.argv[0] + " <pubkey_file> <signature_file> <data_file>")
    sys.exit(-1)

pubKeyBytes = open(sys.argv[1], "rb").read()

print("Public key: " + pubKeyBytes.hex())
pubKey = ed25519.VerifyingKey(pubKeyBytes)

signature = open(sys.argv[2], "rb").read()
msg = open(sys.argv[3], "rb").read()

try:
    pubKey.verify(signature, msg)
    print("Signature verified OK.")
except:
    print("Invalid signature!")
