#include "clipper.hpp"
#include <cstdio>
#include <cstring>
#include <vector>
using namespace ClipperLib;
// read "x,y x,y ..." lines; line prefix P=subject open, C=clip closed
int main(){
  Paths subj, clip; char line[2000000];
  while(fgets(line,sizeof line,stdin)){
    Path p; char*s=line; while(*s&&*s!=' ')++s; // skip tag
    long long x,y; int n;
    while(sscanf(s," %lld,%lld%n",&x,&y,&n)==2){ p<<IntPoint(x,y); s+=n; }
    if(line[0]=='P') subj.push_back(p); else clip.push_back(p);
  }
  Clipper c; c.AddPaths(subj,ptSubject,false); c.AddPaths(clip,ptClip,true);
  PolyTree out; c.Execute(ctIntersection,out,pftNonZero,pftNonZero);
  for(int i=0;i<out.ChildCount();++i){ for(auto&p:out.Childs[i]->Contour) printf("%lld,%lld ",(long long)p.x(),(long long)p.y()); printf("\n"); }
  return 0;
}
