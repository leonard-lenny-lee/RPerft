"""Use Stockfish to debug Perft errors
"""
from enum import Enum
from subprocess import Popen, PIPE
from re import match
from typing import List, Dict

ENGINE_PATH = "./target/debug/chess"
STOCKFISH_PATH = "stockfish"
STARTING_FEN = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"


class DebugResult(Enum):

    OK = 0
    MISSING_MOVE = 1
    EXCESS_MOVE = 2
    DISAGREE_MOVE = 3
    COUNT_MISMATCH = 4

    def load(self, payload):
        self.payload = payload
        return self


def debug(depth: int, fen: str = None):
    if fen is None:
        fen = STARTING_FEN
    moves = []
    for d in range(depth, 0, -1):
        debug_result = _debug(d, fen, moves)
        if debug_result is DebugResult.OK:
            print(debug_result)
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


def _debug(depth: int, fen: str = None, moves: List[str] = None) -> DebugResult:
    chess_perft = _run("engine", depth, fen, moves)
    stockfish_perft = _run("stockfish", depth, fen, moves)
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


def _run(engine: str, depth: int, fen: str = None, moves: List[str] = None) -> Dict[str, int]:
    if engine == "stockfish":
        path = STOCKFISH_PATH
    else:
        path = ENGINE_PATH
    p = Popen(path, stdin=PIPE, stdout=PIPE, encoding="UTF8")
    if moves is not None:
        moves = " ".join(moves)
    if fen is not None:
        # Remove this conditional logic when engine is UCI compliant
        if engine == "stockfish":
            p.stdin.write(f"position fen {fen} moves {moves}\n")
        else:
            p.stdin.write(f"position fen {fen} {moves}\n")
    p.stdin.write(f"go perft {depth}\n")
    p.stdin.write("quit\n")
    p.stdin.flush()
    # Parse response
    response = p.stdout.read().split("\n")
    result = {}
    for line in response:
        if match("^[a-h]{1}[1-8]{1}", line):
            move, node_count = line.split(":")
            result[move] = node_count
    return result


if __name__ == "__main__":
    debug(5, "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
