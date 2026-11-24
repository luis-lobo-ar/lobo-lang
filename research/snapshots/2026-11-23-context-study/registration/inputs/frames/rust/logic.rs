use crate::model::Summary;
struct Header {length:i64,payload:usize,error:i64}
fn read_size(data:&[u8],start:usize)->Header {Header {length:data[start] as i64,payload:start+1,error:0}}
fn size_allowed(length:i64,limit:i64)->bool {length<=limit}
fn payload_checksum(data:&[u8])->u8 {data.iter().fold(7u8,|sum,b|sum.wrapping_add(*b))}
pub fn run(data:&[u8],limit:i64)->Summary {
 let mut frames=Vec::new();let mut cursor=0;let mut error=0;
 while cursor<data.len() && error==0 {
  let header=read_size(data,cursor);
  if header.error!=0 {error=header.error;} else if !size_allowed(header.length,limit) {error=3;}
  else if data.len()-header.payload<(header.length+1) as usize {error=1;}
  else {let end=header.payload+header.length as usize;let payload=&data[header.payload..end];
   if payload_checksum(payload)!=data[end] {error=2;} else {frames.push(payload.to_vec());cursor=end+1;}
  }
 }
 Summary {frames,consumed:cursor as i64,error}
}
