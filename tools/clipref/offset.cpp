// ClipperOffset reference: reads closed polygons "x,y x,y ..." on stdin (one per line),
// offsets by DELTA (env, scaled units), jtRound, prints output polygons.
#include "clipper.hpp"
#include <cstdio>
#include <cstdlib>
#include <cstring>
using namespace ClipperLib;
int main(){
    double delta = atof(getenv("CLIP_DELTA")?getenv("CLIP_DELTA"):"1000");
    double miter = atof(getenv("CLIP_MITER")?getenv("CLIP_MITER"):"3.0");
    double arctol= atof(getenv("CLIP_ARCTOL")?getenv("CLIP_ARCTOL"):"0.0");
    char buf[1<<20];
    ClipperOffset off(miter, arctol);
    int nsub=0;
    while(fgets(buf,sizeof buf,stdin)){
        if(buf[0]=='#'||buf[0]=='\n')continue;
        Path p; char*s=buf; long long x,y;
        while(sscanf(s,"%lld,%lld",&x,&y)==2){ p<<IntPoint(x,y); s=strchr(s,' '); if(!s)break; ++s; }
        if(p.size()>=3){ off.AddPath(p,jtMiter,etClosedPolygon); ++nsub; }
    }
    Paths out; off.Execute(out, delta);
    for(auto&poly:out){ for(auto&pt:poly) printf("%lld,%lld ",(long long)pt.x(),(long long)pt.y()); printf("\n"); }
    return 0;
}
