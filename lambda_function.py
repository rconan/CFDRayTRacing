import numpy as np
import os
import boto3
import urllib
from scipy.interpolate import NearestNDInterpolator, LinearNDInterpolator, interpn, RegularGridInterpolator
from scipy.io import savemat
from scipy.signal import fftconvolve

s3 = boto3.client('s3')

class CFD_Data(object):
    def __init__(self,filename,nH=101,nPx=421,D=26):
        
        print '@(CFD_Data)>> Loading %s...'%filename
        self.OSS_M1_vertex = 3.9
        self.data = np.loadtxt(filename,
                     delimiter=',',skiprows=1)
        self.data[:,4] -= self.OSS_M1_vertex
        print '@(CFD_Data)>> Total number of sample: %d'%self.data.shape[0]
        print '@(CFD_Data)>> Z min/max: %.3f/%.3f meter'%(self.z.min(),self.z.max())
        
        print '@(CFD_Data)>> Nearest interpolation to a gridded mesh ...'
        self.nearest = NearestNDInterpolator(self.data[:,2:],self.ri.ravel())
        zo = np.linspace(self.z.min(),self.z.max(),nH)
        self.resh = zo[1] - zo[0]
        uo = np.linspace(-1,1,nPx)*D/2
        x3d,y3d,z3d = np.meshgrid(uo,uo,zo,indexing='ij',sparse=True)
        ri_gridded = self.nearest(x3d,y3d,z3d)
        
        print '@(CFD_Data)>> Setting the tri-linear interpolator ...'
        self.interpolate = RegularGridInterpolator((uo,uo,zo),ri_gridded)

    def __call__(self,xi,yi,zi,s):
        xyzi = np.stack([xi,yi,zi],xi.ndim)
        ri_i = 1 + self.interpolate(xyzi)
        ds = np.diff(s,1)
        opl = np.sum(np.abs(ds)*ri_i[:,1:],axis=1,dtype=np.float64)
        return opl - np.mean(opl)
    @property
    def ri(self):
        return self.data[:,0][:,None]
    @property
    def T(self):
        return self.data[:,1][:,None]
    @property
    def x(self):
        return self.data[:,2][:,None]
    @property
    def y(self):
        return self.data[:,3][:,None]
    @property
    def z(self):
        return self.data[:,4][:,None]

def rayTrace(cfd,xyz,klm,zmin,zmax,v,nPx,nh=101,method='nearest'):
    
    #print(zmin,zmax) 

    x = xyz[...,0,None]
    y = xyz[...,1,None]
    z = xyz[...,2,None]
    
    k = klm[...,0,None]
    l = klm[...,1,None]
    m = klm[...,2,None]

    #z0 = np.arange(zmin,zmax,resh)[None,...]
    delta = zmax - zmin
    step = delta/(nh-1)
    u = np.arange(nh)
    z0 = step[...,None]*u[None,...]
    z0 += zmin[...,None]
    z0[:,-1] = zmax
    
    s = (z0 - z)/m
    xp = x + k*s
    yp = y + l*s
    zp = z + m*s
    
    #ds = np.abs(np.diff(s,axis=1))
    #opl = ds.sum(axis=1)
    opl = cfd(xp,yp,zp,s)
    if opl.size>1:
        #print(ds.sum(1))
        #opd = cfd(xp,yp,zp,s)
        #opd = s - 
        opd2d = np.ones(nPx**2)*np.NaN
        opd2d[v] = opl
        return opd2d.reshape(nPx,nPx)
    else:
        return opl


def dome_seeing(filename,ceo,D=25.5,resh=0.25,nPx=401):

    cfd = CFD_Data(filename)

    xyz = ceo['xyz']
    klm = ceo['klm']
    v = ceo['v']
    m = ceo['m']
    nPx = ceo['nPx']
    C = ceo['C']
    AW0 = ceo['AW0']

    opl_1 = rayTrace(cfd,xyz[0][v,:],klm[0][v,:],xyz[1][v,2],cfd.z.max()*np.ones(v.sum()),v,nPx)
    opl0_1 = rayTrace(cfd,np.asanyarray([0,0,cfd.z.max()]),np.asanyarray([0,0,-1]),np.asanyarray(0.0),cfd.z.max(),v,nPx)
    opl_2 = rayTrace(cfd,xyz[1][v,:],klm[1][v,:],xyz[1][v,2],xyz[2][v,2],v,nPx)
    opl0_2 = rayTrace(cfd,np.asanyarray([0,0,0]),np.asanyarray([0,0,1]),np.asanyarray(0.0),xyz[2][v,2].max(),v,nPx)
    opl_3 = rayTrace(cfd,xyz[2][v,:],klm[2][v,:],xyz[3][v,2],xyz[2][v,2],v,nPx)
    opl0_3 =  rayTrace(cfd,np.asanyarray([0,0,xyz[2][v,2].max()]),np.asanyarray([0,0,-1]),xyz[3][v,2].max(),xyz[2][v,2].max(),v,nPx)

    opd = opl_1+opl0_1+(opl_2+opl0_2)+(opl_3+opl0_3)
    opd[m.reshape(nPx,nPx)==1] = np.NaN
    opd -= np.nanmean(opd)

    A = opd.copy()
    A[np.isnan(opd)] = 0.0
    A[~np.isnan(opd)] = 1.0
    F = opd.copy()
    F[np.isnan(opd)] = 0.0
    wavelength = 500e-9
    k = 2.*np.pi/wavelength
    W = A*np.exp(1j*k*F)
    S1 = np.fliplr(np.flipud(W))
    S2 = np.conj(W)
    AW = fftconvolve(S1,S2)
    pssn = np.sum(np.abs(AW*C)**2)/np.sum(np.abs(AW0*C)**2)

    print 'PSSNn=%.4f'%pssn

    return {'opd':opd,'PSSn':pssn}

def lambda_handler(event, context):

    for record in event['Records']:
        print record
        key = urllib.unquote(record['s3']['object']['key'])
        print key
        downfile = '/tmp/'+key
        s3.download_file('cfd.scattered',key,downfile)

        ceodatafile = '/tmp/cfdRaytrace.npz'
        if not os.path.isfile(ceodatafile):
            s3.download_file('gmto.rconan','cfdRaytrace.npz',ceodatafile)
        ceo = np.load(ceodatafile)
        data = dome_seeing(downfile,ceo)
        os.remove(downfile)
        filename, file_extension = os.path.splitext(key)
        upkey = 'reduced_'+filename+'.mat'
        upfile='/tmp/'+upkey
        savemat(upfile,data)

        s3.upload_file(upfile,'cfd.gridded',upkey)
        os.remove(upfile)

    return 'Interpolation completed!'

if __name__ == "__main__":
    import json
    f = open("inputFile.json")
    event = json.load(f)
    response_code = lambda_handler(event=event, context=None)
    print(response_code)
