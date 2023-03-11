"""Use Stockfish to debug Perft errors
"""
import argparse
from enum import Enum
from subprocess import Popen, PIPE
from re import match
from typing import List, Dict

ENGINE_PATH = "./target/debug/chess"
STOCKFISH_PATH = "stockfish"
STARTING_FEN = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"


def main():
    parser = argparse.ArgumentParser(
        description="Debug Perft by comparing against a reference engine"
    )
    parser.add_argument("--ref", "-r", metavar="R", type=str, nargs=1,
                        help="path to reference engine", default=STOCKFISH_PATH)
    parser.add_argument("--eng", "-e", metavar="E", type=str, nargs=1,
                        help="path to test engine", default=ENGINE_PATH)
    parser.add_argument("--depth", "-d", metavar="D", type=str, nargs=1,
                        help="depth to search", required=False, default="5")
    parser.add_argument("--fen", "-f", metavar="F", type=str, nargs="+",
                        help="fen string", required=False, default=STARTING_FEN)
    args = parser.parse_args()
    args = map(lambda x: " ".join(x) if isinstance(x, list) else x,
               (args.depth, args.fen, args.ref, args.eng))
    debug(*args)


class DebugResult(Enum):

    OK = 0
    MISSING_MOVE = 1
    EXCESS_MOVE = 2
    DISAGREE_MOVE = 3
    COUNT_MISMATCH = 4

    def load(self, payload):
        self.payload = payload
        return self


def debug(depth: str, fen: str, ref: str, eng: str):
    moves = []
    for d in range(int(depth), 0, -1):
        debug_result = _debug(ref, eng, d, fen, moves)
        if debug_result is DebugResult.OK:
            print(f"{debug_result} {fen} depth {depth}")
            return
        if debug_result is DebugResult.COUNT_MISMATCH:
            # Explore deeper
            moves.append(debug_result.payload)
            continue
        # All other cases, we have found the variation resulting in the error
        variation = fen + " " + " ".join(moves)
        print(f"{debug_result}: {debug_result.payload}\nVariation: {variation}")
        return
    print("Debug failed to find variation")


def _debug(ref: str, eng: str, depth: int, fen: str, moves: List[str] = None) -> DebugResult:
    chess_perft = _run(eng, depth, fen, moves)
    stockfish_perft = _run(ref, depth, fen, moves)
    # Check if there is disagreement in number of, or identity, of moves
    # found in the current position, load mismatch information
    egn_moves, sf_moves = set(chess_perft.keys()), set(stockfish_perft.keys())
    if egn_moves != sf_moves:
        if len(egn_moves) > len(sf_moves):
            return DebugResult.EXCESS_MOVE.load(egn_moves - sf_moves)
        elif len(sf_moves) > len(egn_moves):
            return DebugResult.MISSING_MOVE.load(sf_moves - egn_moves)
        else:
            # Symmetric difference
            return DebugResult.DISAGREE_MOVE.load(egn_moves ^ sf_moves)
    # Scan for node count difference, load culprit variation to further explore
    for k, v in chess_perft.items():
        if stockfish_perft[k] != v:
            return DebugResult.COUNT_MISMATCH.load(k)
    return DebugResult.OK


def _run(path: str, depth: int, fen: str, moves: List[str] = None) -> Dict[str, int]:
    p = Popen(path, stdin=PIPE, stdout=PIPE, encoding="UTF8")
    if moves is not None:
        moves = " ".join(moves)
    p.stdin.write(f"position fen {fen} moves {moves}\n")
    p.stdin.write(f"go perft {depth}\n")
    p.stdin.write("quit\n")
    p.stdin.flush()
    # Parse response
    response = p.stdout.read().split("\n")
    result = {}
    for line in response:
        if match("^[a-h][1-8]{2}", line):
            move, node_count = line.split(":")
            result[move] = node_count
    return result


if __name__ == "__main__":
    main()
