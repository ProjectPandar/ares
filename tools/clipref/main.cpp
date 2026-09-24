#include "clipper.hpp"
#include <cstdio>
using namespace ClipperLib;
int main(){
  Path subject; subject << IntPoint(2,3) << IntPoint(11,5) << IntPoint(4,8);
  Paths clip; Path sq; sq<<IntPoint(0,0)<<IntPoint(10,0)<<IntPoint(10,10)<<IntPoint(0,10); clip<<sq;
  Clipper c; c.AddPath(subject, ptSubject, false); c.AddPaths(clip, ptClip, true);
  PolyTree out; c.Execute(ctIntersection, out, pftNonZero, pftNonZero);
  for(int i=0;i<out.ChildCount();++i){ for(auto&p:out.Childs[i]->Contour) printf("%lld,%lld ",(long long)p.x(),(long long)p.y()); printf("| "); }
  printf("\n"); return 0;
}
