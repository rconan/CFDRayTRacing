import numpy as np
import os
import boto3
import urllib
from scipy.interpolate import NearestNDInterpolator, griddata
from scipy.io import savemat

s3 = boto3.client('s3')

def dome_seeing(filename,D=25.5,resh=0.25,nPx=401):

    data = np.loadtxt(filename,
                         delimiter=',',skiprows=1)

    xyz = data[:,-3:]
    ri = data[:,0]

    nearest = NearestNDInterpolator(xyz,ri)

    zo = np.arange(4,54+resh,resh)
    uo = np.linspace(-1,1,nPx)*D/2
    x3d,y3d,z3d = np.meshgrid(uo,uo,zo)
    
    rio = nearest(x3d,y3d,z3d)

    return {'D':D,'resh':resh,'nPx':nPx,'uo':uo,'zo':zo,'rio':rio}    

def lambda_handler(event, context):

    for record in event['Records']:
        print record
        key = urllib.unquote(record['s3']['object']['key'])
        print key
        downfile = '/tmp/'+key
        s3.download_file('cfd.scattered',key,downfile)

        data = dome_seeing(downfile)
        os.remove(downfile)
        filename, file_extension = os.path.splitext(key)
        upkey = filename+'.mat'
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
