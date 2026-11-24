from dataclasses import dataclass
@dataclass(frozen=True)
class Job:
 id:int
 cost:int
 deps:list[int]
@dataclass
class Summary:
 order:list[int]
 spent:int
 blocked:int
