from dataclasses import dataclass
@dataclass
class Summary:
 frames:list[bytes]
 consumed:int
 error:int
