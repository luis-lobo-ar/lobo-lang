from model import Summary

def read_size(data,start):
 return data[start],start+1,0

def size_allowed(length,limit):
 return length<=limit

def payload_checksum(data):
 return (7+sum(data))%256

def run(data,limit):
 frames=[];cursor=0;error=0
 while cursor<len(data) and error==0:
  length,payload,header_error=read_size(data,cursor)
  if header_error:error=header_error
  elif not size_allowed(length,limit):error=3
  elif len(data)-payload<length+1:error=1
  else:
   value=data[payload:payload+length]
   if payload_checksum(value)!=data[payload+length]:error=2
   else:frames.append(value);cursor=payload+length+1
 return Summary(frames,cursor,error)
