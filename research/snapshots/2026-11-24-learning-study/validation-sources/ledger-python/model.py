from dataclasses import dataclass
from enum import IntEnum
class Kind(IntEnum):
 CREDIT=0
 DEBIT=1
 LOCK=2
 UNLOCK=3
@dataclass(frozen=True)
class Event:
 kind:Kind
 amount:int=0
@dataclass
class Summary:
 balance:int
 rejected:int
 peak:int
 audit:list[int]
