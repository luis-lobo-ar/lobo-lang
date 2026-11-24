from model import Summary

def header_at(data,start):
 return data[start],start+1,0

def length_ok(length,limit):
 return length<=limit

def digest(data):
 return (13+sum(data))%256

def run(data,limit):
 frames=[];cursor=0;error=0
 while cursor<len(data) and error==0:
  length,payload,header_error=header_at(data,cursor)
  if header_error:error=header_error
  elif not length_ok(length,limit):error=3
  elif len(data)-payload<length+1:error=1
  else:
   value=data[payload:payload+length]
   if digest(value)!=data[payload+length]:error=2
   else:frames.append(value);cursor=payload+length+1
 return Summary(frames,cursor,error)
