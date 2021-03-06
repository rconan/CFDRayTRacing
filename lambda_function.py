import numpy as np
import os
import boto3
import urllib
from scipy.interpolate import NearestNDInterpolator, LinearNDInterpolator, interpn, RegularGridInterpolator
from scipy.io import savemat
from scipy.signal import fftconvolve
import gzip
from io import StringIO, BytesIO
import botocore

#s3 = boto3.client('s3')

class CFD_Data(object):
    def __init__(self,raw_data,wlm=0.5,nH=81,nPx=43,D=26.25):

        self.wlm = wlm
        self.OSS_M1_vertex = 3.9
        if raw_data.shape[1]>4:
            raw_data = np.delete(raw_data,1,1)
        self.data = raw_data
        self.data[:,-1] -= self.OSS_M1_vertex
        ##$print('@(CFD_Data)>> Total number of sample: %d'%self.data.shape[0])
        ##$print('@(CFD_Data)>> Z min/max: %.3f/%.3f meter'%(self.z.min(),self.z.max()))
        
        #$print('@(CFD_Data)>> Nearest interpolation to a gridded mesh ...')
        nearest = NearestNDInterpolator(self.data[:,1:],self.ri.ravel())
        self.zo = np.linspace(self.z.min(),self.z.max(),nH)
        self.resh = self.zo[1] - self.zo[0]
        self.uo = np.linspace(-1,1,nPx)*D/2
        x3d,y3d,z3d = np.meshgrid(self.uo,self.uo,self.zo,indexing='ij',sparse=True)
        ri_gridded = nearest(x3d,y3d,z3d)
        
        #$print('@(CFD_Data)>> Setting the tri-linear interpolator ...')
        self.interpolate = RegularGridInterpolator((self.uo,self.uo,self.zo),
                                                   ri_gridded,
                                                   bounds_error=False,
                                                   fill_value = None)

    def __call__(self,xi,yi,zi,s):
        xyzi = np.stack([xi,yi,zi],xi.ndim)
        ri_i = self.interpolate(xyzi)
        ds = np.diff(s,1)
        opl = np.sum(np.abs(ds)*ri_i[:,1:],axis=1,dtype=np.float64)
        return opl
    @property
    def ri(self):
        pref = 75000.0 # Reference pressure
        tp = self.T
        _ri_ = 7.76e-7*pref*(1+0.00752/self.wlm**2)/tp # create refractive index from temperature
        return _ri_
    @property
    def T(self):
        return self.data[:,0][:,None]
    @property
    def x(self):
        return self.data[:,1][:,None]
    @property
    def y(self):
        return self.data[:,2][:,None]
    @property
    def z(self):
        return self.data[:,3][:,None]

def PSSn(opd,wlm,C,AW0):
    A = opd.copy()
    A[np.isnan(opd)] = 0.0
    A[~np.isnan(opd)] = 1.0
    F = opd.copy()
    F[np.isnan(opd)] = 0.0
    k = 2e6*np.pi/wlm
    W = A*np.exp(1j*k*F)
    S1 = np.fliplr(np.flipud(W))
    S2 = np.conj(W)
    AW = fftconvolve(S1,S2)
    pssn = np.sum(np.abs(AW*C)**2)/np.sum(np.abs(AW0*C)**2)
    return pssn

def rayTrace(cfd,params,nH=81,cfd_nPx=43):

    N_RAY = params['xyz0'].shape[0]
    #print('Ray tracing (N_RAY=%d):'%N_RAY)
    cfd_opd = np.ones(N_RAY)*np.nan
    #v123 = params['v1']*params['v2']*params['v3']*params['m']
    v123 = params['m']

    a = 0
    step = 200000
    b = step
    n_step = int(np.ceil(N_RAY/step))
    for k in range(n_step):

        #$print('. Rays range: [%d,%d]'%(a,b))
        _ = np.s_[a:b]

        #$print(' . Source to M1')
        v = v123[_]
        xyz_0 = params['xyz0'][_,:][v,:]
        klm_0 = params['klm0'][_,:][v,:]
        xyz_1 = params['xyz1'][_,:][v,:]

        s_range = (xyz_1[:,-1] - xyz_0[:,-1])/klm_0[:,-1]/(nH-1)
        u = np.arange(nH)
        s = s_range[...,None]*u[None,...]

        x = xyz_0[:,0][...,np.newaxis] + klm_0[:,0][...,np.newaxis]*s
        y = xyz_0[:,1][...,np.newaxis] + klm_0[:,1][...,np.newaxis]*s
        z = xyz_0[:,2][...,np.newaxis] + klm_0[:,2][...,np.newaxis]*s

        cfd_opd1 = cfd(x,y,z,s)

        #$print(' . M1 to M2')
        xyz_0 = params['xyz1'][_,:][v,:]
        klm_0 = params['klm1'][_,:][v,:]
        xyz_1 = params['xyz2'][_,:][v,:]

        s_range = (xyz_1[:,-1] - xyz_0[:,-1])/klm_0[:,-1]/(nH-1)
        u = np.arange(nH)
        s = s_range[...,None]*u[None,...]

        x = xyz_0[:,0][...,np.newaxis] + klm_0[:,0][...,np.newaxis]*s
        y = xyz_0[:,1][...,np.newaxis] + klm_0[:,1][...,np.newaxis]*s
        z = xyz_0[:,2][...,np.newaxis] + klm_0[:,2][...,np.newaxis]*s

        cfd_opd2 = cfd(x,y,z,s)

        #$print(' . M2 to exit pupil')
        xyz_0 = params['xyz2'][_,:][v,:]
        klm_0 = params['klm2'][_,:][v,:]
        xyz_1 = params['xyz3'][_,:][v,:]

        s_range = (xyz_1[:,-1] - xyz_0[:,-1])/klm_0[:,-1]/(nH-1)
        u = np.arange(nH)
        s = s_range[...,None]*u[None,...]

        x = xyz_0[:,0][...,np.newaxis] + klm_0[:,0][...,np.newaxis]*s
        y = xyz_0[:,1][...,np.newaxis] + klm_0[:,1][...,np.newaxis]*s
        z = xyz_0[:,2][...,np.newaxis] + klm_0[:,2][...,np.newaxis]*s

        cfd_opd3 = cfd(x,y,z,s)

        cfd_opd[_][v] = cfd_opd1+cfd_opd2+cfd_opd3

        a = b
        b = np.minimum(b+step,N_RAY)
        
    cfd_opd -= np.nanmean(cfd_opd)

    return {'opd':cfd_opd,'opd max':np.nanmax(cfd_opd),'opd min':np.nanmin(cfd_opd)}#,'V_PSSn':V_PSSn,'H_PSSn':H_PSSn}

"""
    n = int(np.sqrt(N_RAY))
    V_PSSn = PSSn(cfd_opd.reshape(n,n),0.5,params['V_C'],params['V_AW0'])
    H_PSSn = PSSn(cfd_opd.reshape(n,n),1.65,params['H_C'],params['H_AW0'])
    print('PSSn:',V_PSSn,H_PSSn)
"""

def lambda_handler(event, context):

    for record in event['Records']:

        #print(record)
        bucket = urllib.parse.unquote(record['s3']['bucket']['name'])
        key = urllib.parse.unquote(record['s3']['object']['key'])
        s3 = boto3.resource('s3')

        filename, file_extension = os.path.splitext(key)
        filename, file_extension = os.path.splitext(filename)
        upkey = filename+'.npz'

        try:
            s3.Object(bucket, upkey).load()
        except botocore.exceptions.ClientError as e:
            if e.response['Error']['Code'] == "404":
                print(f"Downloading {key}")
                raw_data = np.loadtxt(StringIO(gzip.decompress(s3.Object(bucket, key).get()['Body'].read()).decode('utf-8')),delimiter=',',skiprows=1)
                print(f"Gridding dome seeing")
                cfd = CFD_Data(raw_data,nH=101,nPx=525)

                print(f"Downloading ray tracing parameter")
                ceo = np.load(BytesIO(s3.Object('cfd.archive','gs_onaxis_params_512.npz').get()['Body'].read()))
                print("Ray tracing")
                data = rayTrace(cfd,ceo,nH=101)

                buf = BytesIO()
                np.savez(buf,**data)
                buf.seek(0)
                print(f"Uploading {upkey}")
                s3.Object(bucket, upkey).put(Body=buf.read())
                return 'Interpolation completed!'
            else:
                raise
        else:
            return 'The object does exist!'

if __name__ == "__main__":
    import json
    f = open("inputFile.json")
    event = json.load(f)
    response_code = lambda_handler(event=event, context=None)
    print(response_code)
