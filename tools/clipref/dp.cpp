#include "clipper.hpp"
#include <cstdio>
#include <vector>
#include <cmath>
using namespace ClipperLib;
// replicate MultiPoint::_douglas_peucker
std::vector<IntPoint> dp(const std::vector<IntPoint>&pts,double tol){
  std::vector<IntPoint> res; double tsq=tol*tol;
  if(pts.empty())return res;
  size_t a=0,f=pts.size()-1; res.push_back(pts[a]);
  if(a==f)return res; std::vector<size_t> st{f};
  for(;;){double mx=0;size_t fi=a;
    for(size_t i=a+1;i<f;++i){ // distance_to_squared
      double vx=pts[f].x()-pts[a].x(),vy=pts[f].y()-pts[a].y();
      double wx=pts[i].x()-pts[a].x(),wy=pts[i].y()-pts[a].y();
      double l2=vx*vx+vy*vy,d;
      if(l2==0)d=wx*wx+wy*wy;else{double t=(wx*vx+wy*vy)/l2;
        if(t<=0)d=wx*wx+wy*wy;else if(t>=1){double ex=pts[i].x()-pts[f].x(),ey=pts[i].y()-pts[f].y();d=ex*ex+ey*ey;}
        else{double dx=t*vx-wx,dy=t*vy-wy;d=dx*dx+dy*dy;}}
      if(d>mx){mx=d;fi=i;}}
    if(mx<=tsq){res.push_back(pts[f]);a=f;st.pop_back();if(st.empty())break;f=st.back();}
    else{f=fi;st.push_back(f);}}
  return res;}
int main(){std::vector<IntPoint>pts;long long x,y;char line[100000];
  while(fgets(line,sizeof line,stdin)){char*s=line;int n;while(sscanf(s," %lld,%lld%n",&x,&y,&n)==2){pts.push_back(IntPoint(x,y));s+=n;}}
  auto r=dp(pts,12000.0); // scaled(0.012)
  for(auto&p:r)printf("%lld,%lld ",(long long)p.x(),(long long)p.y());printf("\n");
  fprintf(stderr,"in=%zu out=%zu\n",pts.size(),r.size());}
